# ADR-014: Multi-Currency Salary

| Property       | Value                                                       |
|----------------|-------------------------------------------------------------|
| **Status**     | Proposed                                                    |
| **Date**       | 2026-08-03                                                  |
| **Author**     | AI + Lee                                                    |
| **Depends on** | [ADR-003](ADR-003-payroll-formula-engine.md), [ADR-006](ADR-006-accumulators-multi-period-context.md) |
| **Domain**     | Payroll — Currency Handling                                 |

---

## 1. Context

### 1.1 The FDI Reality

Many companies in Vietnam pay salaries in or pegged to foreign currencies:

| Pattern | Example | Who Uses It |
|---------|---------|-------------|
| **Pegged to FX** | "Lương = 2000 USD, quy đổi ra VND theo tỷ giá ngày tính lương" | FDI companies, tech startups, expat packages |
| **Dual currency** | Base salary in USD, allowances in VND | Senior management, expatriates |
| **Full foreign currency** | Entire salary in USD, transferred to foreign account | Expats (legal in VN since Circular 32/2013) |
| **Multi-currency components** | Each component can have its own currency | Complex compensation structures |

### 1.2 The Core Problem

All calculations in ADR-003 through ADR-013 assume VND (`Decimal`). But with multi-currency:

```
Gross Salary = 2000 USD @ 24,500 VND/USD = 49,000,000 VND
Transport Allowance = 3,000,000 VND
Housing Allowance = 500 USD @ 24,500 VND/USD = 12,250,000 VND
─────────────────────────────────────────────────
BHXH base = 49,000,000 + 3,000,000 + 12,250,000 = 64,250,000 VND
  → BHXH = 64,250,000 × 8% = 5,140,000 VND

Net pay in USD = (64,250,000 - 5,140,000 - PIT_VND) / 24,500 = X USD
```

Key observations:
- **Insurance and tax are ALWAYS in VND** (Vietnamese law)
- **FX rates matter for every conversion**
- **Employees may receive net pay in a different currency than their gross**

---

## 2. Architecture Decision

### 2.1 Single Internal Currency: VND

All calculations happen in VND. Currency conversion is an **input/output concern**, not a calculation concern:

```
┌─────────────────────────────────────────────────────────────┐
│                  CURRENCY CONVERSION BOUNDARY                 │
│                                                              │
│  INPUT: CurrencyAware salary components                      │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ "Basic Salary: 2000 USD"  ──→ convert to VND ──→ 49M  │   │
│  │ "Housing: 500 USD"        ──→ convert to VND ──→ 12.3M│   │
│  │ "Transport: 3,000,000 VND"──→ already VND ────→ 3M    │   │
│  └──────────────────────────────────────────────────────┘   │
│                         │                                     │
│                         ▼                                     │
│  ┌──────────────────────────────────────────────────────┐   │
│  │        ADR-003 PIPELINE (all VND internally)          │   │
│  │  Gross 64.3M → BHXH 5.1M → PIT 2M → Net 57.2M (VND) │   │
│  └──────────────────────────────────────────────────────┘   │
│                         │                                     │
│                         ▼                                     │
│  OUTPUT: Net pay in preferred currency                        │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ Net: 57,200,000 VND  ──→ convert ──→ 2,334 USD        │   │
│  │   OR Net: 57,200,000 VND (if employee prefers VND)     │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

This avoids polluting the calculation engine with currency logic. Every component receives VND, outputs VND. The `VariableContext` is purely VND.

### 2.2 Currency-Aware Salary Components

```sql
ALTER TABLE employee_salary_components
    ADD COLUMN currency TEXT NOT NULL DEFAULT 'VND',
    ADD COLUMN fx_rate_source TEXT DEFAULT 'manual'
        CHECK (fx_rate_source IN ('manual','vietcombank_bid','vietcombank_ask',
                'state_bank','custom_provider'));

-- USD-denominated component example:
-- employment_id=X, amount=2000, currency='USD', fx_rate_source='vietcombank_bid'
```

### 2.3 Exchange Rate Provider

```rust
/// Exchange rate resolution — a VariableProvider (ADR-003)
trait ExchangeRateProvider: Send + Sync {
    fn provider_id(&self) -> &str;

    /// Get the VND exchange rate for a currency on a specific date
    fn get_rate(&self, currency: &str, date: NaiveDate) -> Result<Decimal, FxError>;

    /// Which currencies this provider supports
    fn supported_currencies(&self) -> Vec<String>;
}

struct VietcombankRateProvider {
    api_endpoint: String,
    cache: Arc<RwLock<HashMap<(String, NaiveDate), Decimal>>>,
}

impl ExchangeRateProvider for VietcombankRateProvider {
    fn get_rate(&self, currency: &str, date: NaiveDate) -> Result<Decimal, FxError> {
        // Check cache first
        if let Some(rate) = self.cache.read().get(&(currency.to_string(), date)) {
            return Ok(*rate);
        }
        // Fetch from VCB API or fallback to stored rate
        // ...
    }
}
```

Rate source is configurable per workspace:
- **Manual**: Admin enters rate manually (simplest, most common)
- **Vietcombank**: Auto-fetch from VCB's published rates
- **State Bank**: Central bank reference rate (for regulatory reporting)
- **Custom**: External API/webhook for crypto/other currencies

### 2.4 Rate Resolution at Calculation Time

```rust
impl CurrencyConverter {
    /// Convert a salary component to VND for the calculation pipeline
    pub fn to_vnd(
        component: &EmployeeSalaryComponent,
        period_date: NaiveDate,  // date within the payroll period
    ) -> Result<Decimal, FxError> {
        if component.currency == "VND" {
            return Ok(component.amount);
        }
        let rate = self.rate_provider.get_rate(&component.currency, period_date)?;
        Ok(component.amount * rate)
    }

    /// Convert net pay from VND to employee's preferred currency
    pub fn from_vnd(
        amount_vnd: Decimal,
        target_currency: &str,
        rate_date: NaiveDate,
    ) -> Result<Decimal, FxError> {
        if target_currency == "VND" {
            return Ok(amount_vnd);
        }
        let rate = self.rate_provider.get_rate(target_currency, rate_date)?;
        Ok(amount_vnd / rate)
    }
}
```

### 2.5 Rate Versioning

Rates change daily. For auditability:

```sql
CREATE TABLE payroll_exchange_rates (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL,
    currency TEXT NOT NULL,
    rate_date DATE NOT NULL,
    rate_to_vnd DECIMAL NOT NULL,
    source TEXT NOT NULL,              -- 'manual','vietcombank','state_bank'
    created_by UUID,
    created_at TIMESTAMPTZ DEFAULT now(),
    UNIQUE (workspace_id, currency, rate_date)
);
```

When a period is committed, the exchange rate used is frozen with the payslip. If the period is recalculated later, the SAME rate is used (point-in-time consistency — same principle as template versioning in ADR-003).

### 2.6 Accumulators Are Always VND

Accumulators (ADR-006) track in VND regardless of source currency. This is correct because:
- BHXH cap is defined in VND (20 × regional minimum wage in VND)
- PIT brackets are in VND
- All statutory calculations are in VND

### 2.7 Employee Currency Preference

```sql
ALTER TABLE employments
    ADD COLUMN preferred_currency TEXT DEFAULT 'VND';
```

This controls:
- Which currency the payslip displays net pay in
- Which currency the employee receives (if different from VND)
- Bank transfer currency (ADR-013 supports multi-currency bank instructions)

---

## 3. Risks & Trade-offs

| Risk | Mitigation |
|------|-----------|
| **Rate volatility between calculation and payment** | Rate is **locked at period commit date**. If actual payment rate differs, adjustment is manual. |
| **Crypto/stablecoin salary** | Not directly supported. Custom rate provider can map any currency code to VND via external API. |
| **Dual-currency payslip** | Payslip shows both currencies: "Net: 57,200,000 VND (≈ 2,334 USD @ 24,500)" |
| **Retro with different rate** | Retro always uses the ORIGINAL rate frozen at original calculation time |

---

## 4. References

- [ADR-003](ADR-003-payroll-formula-engine.md) — VariableProvider, pipeline purity
- [ADR-006](ADR-006-accumulators-multi-period-context.md) — VND-only accumulators
- [ADR-011](ADR-011-payslip-generation-reporting.md) — Dual-currency payslip display
- Circular 32/2013/TT-NHNN — Foreign currency salary regulations in Vietnam
