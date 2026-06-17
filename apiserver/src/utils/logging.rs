use std::collections::HashMap;
use std::fmt;
use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Record};
use tracing::{Event, Id, Subscriber};
use tracing_subscriber::fmt::{format::Writer, FormatEvent, FormatFields};
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;

use crate::utils::masking::{mask_bank_account, mask_email, mask_phone};

// ── PII Masking Helper ──────────────────────────────────────────────

fn mask_pii_value(key: &str, val: &str) -> String {
    let key_lower = key.to_lowercase();
    if key_lower.contains("email") {
        mask_email(val)
    } else if key_lower.contains("phone") {
        mask_phone(val)
    } else if key_lower.contains("bank_account") || key_lower.contains("account_number") {
        mask_bank_account(val)
    } else if key_lower.contains("password")
        || key_lower.contains("hash")
        || key_lower.contains("secret")
        || key_lower.contains("token")
        || key_lower.contains("jwt")
        || key_lower.contains("key")
        || key_lower.contains("national_id")
    {
        "[REDACTED]".to_string()
    } else {
        val.to_string()
    }
}

// ── Span Context Propagation Layer ──────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct SpanContext {
    pub request_id: Option<String>,
    pub workspace_id: Option<String>,
    pub actor_id: Option<String>,
}

pub struct SpanContextLayer;

impl<S> Layer<S> for SpanContextLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let Some(span) = ctx.span(id) else { return };
        let mut extensions = span.extensions_mut();

        let mut context = SpanContext::default();
        if let Some(parent) = span.parent() {
            if let Some(parent_ctx) = parent.extensions().get::<SpanContext>() {
                context = parent_ctx.clone();
            }
        }

        struct ContextVisitor<'a>(&'a mut SpanContext);
        impl Visit for ContextVisitor<'_> {
            fn record_str(&mut self, field: &Field, value: &str) {
                if field.name() == "request_id" {
                    self.0.request_id = Some(value.to_string());
                } else if field.name() == "workspace_id" {
                    self.0.workspace_id = Some(value.to_string());
                } else if field.name() == "actor_id" {
                    self.0.actor_id = Some(value.to_string());
                }
            }
            fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
                let value_str = format!("{value:?}");
                if field.name() == "request_id" {
                    self.0.request_id = Some(value_str);
                } else if field.name() == "workspace_id" {
                    self.0.workspace_id = Some(value_str);
                } else if field.name() == "actor_id" {
                    self.0.actor_id = Some(value_str);
                }
            }
        }

        attrs.record(&mut ContextVisitor(&mut context));
        extensions.insert(context);
    }

    fn on_record(&self, id: &Id, values: &Record<'_>, ctx: Context<'_, S>) {
        let Some(span) = ctx.span(id) else { return };
        let mut extensions = span.extensions_mut();
        if let Some(context) = extensions.get_mut::<SpanContext>() {
            struct ContextVisitor<'a>(&'a mut SpanContext);
            impl Visit for ContextVisitor<'_> {
                fn record_str(&mut self, field: &Field, value: &str) {
                    if field.name() == "request_id" {
                        self.0.request_id = Some(value.to_string());
                    } else if field.name() == "workspace_id" {
                        self.0.workspace_id = Some(value.to_string());
                    } else if field.name() == "actor_id" {
                        self.0.actor_id = Some(value.to_string());
                    }
                }
                fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
                    let value_str = format!("{value:?}");
                    if field.name() == "request_id" {
                        self.0.request_id = Some(value_str);
                    } else if field.name() == "workspace_id" {
                        self.0.workspace_id = Some(value_str);
                    } else if field.name() == "actor_id" {
                        self.0.actor_id = Some(value_str);
                    }
                }
            }
            values.record(&mut ContextVisitor(context));
        }
    }
}

// ── Custom Production JSON Formatter with PII Masking ───────────────

struct PiiMaskingVisitor {
    fields: HashMap<String, serde_json::Value>,
    message: Option<String>,
}

impl Visit for PiiMaskingVisitor {
    fn record_str(&mut self, field: &Field, value: &str) {
        let name = field.name();
        if name == "message" {
            self.message = Some(value.to_string());
        } else {
            let masked = mask_pii_value(name, value);
            self.fields
                .insert(name.to_string(), serde_json::Value::String(masked));
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        let name = field.name();
        let value_str = format!("{value:?}");
        if name == "message" {
            self.message = Some(value_str);
        } else {
            let masked = mask_pii_value(name, &value_str);
            self.fields
                .insert(name.to_string(), serde_json::Value::String(masked));
        }
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.fields.insert(
            field.name().to_string(),
            serde_json::Value::Number(value.into()),
        );
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.fields.insert(
            field.name().to_string(),
            serde_json::Value::Number(value.into()),
        );
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.fields
            .insert(field.name().to_string(), serde_json::Value::Bool(value));
    }
}

pub struct ProductionJsonFormatter;

impl<S, N> FormatEvent<S, N> for ProductionJsonFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &tracing_subscriber::fmt::FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let mut visitor = PiiMaskingVisitor {
            fields: HashMap::new(),
            message: None,
        };
        event.record(&mut visitor);

        let metadata = event.metadata();
        let timestamp = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

        let mut request_id = None;
        let mut workspace_id = None;
        let mut actor_id = None;
        let mut span_id = None;

        if let Some(span) = ctx.lookup_current() {
            span_id = Some(span.id().into_u64().to_string());
            for parent in span.scope() {
                if let Some(context) = parent.extensions().get::<SpanContext>() {
                    if request_id.is_none() && context.request_id.is_some() {
                        request_id.clone_from(&context.request_id);
                    }
                    if workspace_id.is_none() && context.workspace_id.is_some() {
                        workspace_id.clone_from(&context.workspace_id);
                    }
                    if actor_id.is_none() && context.actor_id.is_some() {
                        actor_id.clone_from(&context.actor_id);
                    }
                }
            }
        }

        let level_str = metadata.level().to_string();
        let message = visitor.message.unwrap_or_default();

        let mut log_entry = serde_json::json!({
            "timestamp": timestamp,
            "level": level_str,
            "message": message,
            "service": "apiserver",
        });

        if let Some(log_obj) = log_entry.as_object_mut() {
            if let Some(rid) = request_id {
                log_obj.insert("request_id".to_string(), serde_json::Value::String(rid));
            }
            if let Some(wid) = workspace_id {
                log_obj.insert("workspace_id".to_string(), serde_json::Value::String(wid));
            }
            if let Some(aid) = actor_id {
                log_obj.insert("actor_id".to_string(), serde_json::Value::String(aid));
            }
            if let Some(sid) = span_id {
                log_obj.insert("span_id".to_string(), serde_json::Value::String(sid));
            }

            // If level is ERROR, extract error type
            if level_str == "ERROR" {
                if let Some(err_val) = visitor
                    .fields
                    .get("error")
                    .or_else(|| visitor.fields.get("error.type"))
                {
                    log_obj.insert("error.type".to_string(), err_val.clone());
                }
            }

            if !visitor.fields.is_empty() {
                log_obj.insert(
                    "fields".to_string(),
                    serde_json::to_value(visitor.fields).unwrap_or(serde_json::Value::Null),
                );
            }
        }

        if let Ok(serialised) = serde_json::to_string(&log_entry) {
            writeln!(writer, "{serialised}")?;
        }

        Ok(())
    }
}

// ── Unit Tests ──────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_pii_value() {
        assert_eq!(
            mask_pii_value("email", "john.doe@example.com"),
            "joh***@example.com"
        );
        assert_eq!(mask_pii_value("password", "supersecret123"), "[REDACTED]");
        assert_eq!(mask_pii_value("user_phone", "0987654321"), "098****321");
        assert_eq!(mask_pii_value("normal_field", "value"), "value");
    }
}
