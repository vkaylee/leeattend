#!/usr/bin/env python3
"""Generate Go-owned OpenAPI route skeleton from current chi registrations.

Route inventory is authoritative. Schema definitions must be enriched from Go DTO
contracts before publishing a client contract; generic objects keep generation
compilable while domain slices add field-level accuracy.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ROUTE_FILES = sorted((ROOT / "apiserver").glob("routes*.go"))
REGISTER = re.compile(r'registerRoute\([^,]+,\s*"([A-Z]+)\s+([^\"]+)"')
DIRECT = re.compile(r'router\.Get\("([^\"]+)",')
SCHEMA_REF = re.compile(r'components\["schemas"\]\["([A-Za-z0-9_]+)"\]')


def route_inventory() -> list[tuple[str, str]]:
    routes: set[tuple[str, str]] = set()
    for path in ROUTE_FILES:
        text = path.read_text(encoding="utf-8")
        routes.update((method, route) for method, route in REGISTER.findall(text))
        routes.update(("GET", route) for route in DIRECT.findall(text))
    return sorted(routes)


def operation_id(method: str, path: str) -> str:
    value = re.sub(r"[^a-zA-Z0-9]+", "_", path).strip("_")
    return f"{method.lower()}_{value or 'root'}"


def openapi_path(path: str) -> str:
    return re.sub(r"\{[^{}]+\}", lambda match: match.group(0), path)


FORBIDDEN_SCHEMA = re.compile(
    r"^(?:Workspace|Tenant|SystemRole|DbCluster)|(?:Workspace|Tenant|SystemRole|DbCluster)|"
    r"(?:CreateTenant|CreateWorkspace|UpdateTenant|UpdateWorkspace)|"
    r"(?:TenantConnection|TenantStats|SystemTenant|SystemRole)",
)


ROUTE_REQUEST_SCHEMAS = {
    ("POST", "/api/v1/employees/match"): "MatchEmployeeRequest",
    ("POST", "/api/v1/employees/{employee_id}/tenures"): "CreateEmployeeTenureRequest",
    ("POST", "/api/v1/employees/{employee_id}/tenures/{tenure_id}/close"): "CloseEmployeeTenureRequest",
    ("POST", "/api/v1/employees/{employee_id}/employments/{employment_id}/terminate"): "TerminateEmploymentRequest",
    ("POST", "/api/v1/files/presigned"): "CreateUploadUrlRequest",
}

ROUTE_MULTIPART_REQUESTS = {
    ("POST", "/api/v1/files/upload"),
}

ROUTE_RESPONSE_SCHEMAS = {
    ("POST", "/api/v1/employees/match"): ("200", "MatchEmployeeResponse"),
    ("GET", "/api/v1/employees/{employee_id}/tenures"): ("200", "EmployeeTenureRecord"),
    ("POST", "/api/v1/employees/{employee_id}/tenures"): ("201", "EmployeeTenureCreated"),
    ("POST", "/api/v1/employees/{employee_id}/tenures/{tenure_id}/close"): ("200", "EmployeeTenureClosed"),
    ("POST", "/api/v1/employees/{employee_id}/employments/{employment_id}/terminate"): ("200", "TerminateEmploymentResult"),
    ("POST", "/api/v1/files/presigned"): ("200", "CreateUploadUrlResponse"),
    ("POST", "/api/v1/files/upload"): ("201", "FileRecord"),
}

REQUIRED_SCHEMA_FIELDS = {
    "CreateUploadUrlRequest": ("category", "file_name", "file_size", "mime_type", "owner_type"),
    "CreateUploadUrlResponse": ("file", "upload_url"),
}

EXPLICIT_SCHEMAS: dict[str, list[str]] = {
    "EmployeeMatchCandidate": [
        "    EmployeeMatchCandidate:",
        "      type: object",
        "      additionalProperties: false",
        "      properties:",
        "        display_name:",
        "          type: string",
        "        employee_code:",
        "          type: string",
        "        employee_id:",
        "          type: string",
        "          format: uuid",
        "        reasons:",
        "          type: array",
        "          items:",
        "            type: string",
        "            enum:",
        "              - national_id_exact",
        "              - phone_exact",
        "              - name_dob_review",
        "              - name_phone_review",
        "        status:",
        "          type: string",
        "          enum:",
        "            - active",
        "            - terminated",
        "        tier:",
        "          type: string",
        "          enum:",
        "            - hard",
        "            - soft",
        "      required:",
        "        - display_name",
        "        - employee_code",
        "        - employee_id",
        "        - reasons",
        "        - status",
        "        - tier",
    ],
    "MatchEmployeeResponse": [
        "    MatchEmployeeResponse:",
        "      type: object",
        "      additionalProperties: false",
        "      properties:",
        "        candidates:",
        "          type: array",
        "          items:",
        "            $ref: '#/components/schemas/EmployeeMatchCandidate'",
        "        tier:",
        "          type: string",
        "          enum:",
        "            - hard",
        "            - soft",
        "            - none",
        "      required:",
        "        - candidates",
        "        - tier",
    ],
}

LIFECYCLE_GUARDED_ROUTES = {
    # Scheduling legacy mutation guards (409 blocked / 503 schema unavailable)
    ("PUT", "/api/v1/shifts/{shift_id}"),
    ("POST", "/api/v1/shifts/assign"),
    ("PUT", "/api/v1/working-patterns/{pattern_id}"),
    ("DELETE", "/api/v1/working-patterns/{pattern_id}"),
    ("POST", "/api/v1/employments/{employment_id}/working-patterns"),
    ("POST", "/api/v1/working-patterns/bulk-assign"),
    ("POST", "/api/v1/employees/{id}/rosters"),
    ("POST", "/api/v1/employees/{id}/rosters/bulk"),
    ("DELETE", "/api/v1/employees/{id}/rosters/{date}"),
    ("PUT", "/api/v1/shift-swaps/{id}/approve"),
    ("POST", "/api/v1/work-segments"),
    ("POST", "/api/v1/work-segments/bulk"),
    ("PUT", "/api/v1/work-segments/{segment_id}"),
    ("DELETE", "/api/v1/work-segments/{segment_id}"),
    # Assignment rules & employee assignments
    ("POST", "/api/v1/assignment-rules"),
    ("PATCH", "/api/v1/assignment-rules/{rule_id}"),
    ("DELETE", "/api/v1/assignment-rules/{rule_id}"),
    ("POST", "/api/v1/employees/{employee_id}/assignments"),
    ("PATCH", "/api/v1/employees/{employee_id}/assignments/{assignment_id}"),
    ("DELETE", "/api/v1/employees/{employee_id}/assignments/{assignment_id}"),
    # Attendance adjustments, time entries, paper OT, adjustment upload
    ("PUT", "/api/v1/attendance/adjustments/{id}/approve"),
    ("PUT", "/api/v1/time/entries/{id}"),
    ("DELETE", "/api/v1/time/entries/{id}"),
    ("POST", "/api/v1/attendance/enter-paper-ot/{employee_id}"),
    ("POST", "/api/v1/attendance/paper-forms/overtime/{employee_id}"),
    ("POST", "/api/v1/attendance/upload"),
    # Holidays
    ("POST", "/api/v1/holidays/{id}/process-leave"),
}

SCHEDULE_LIFECYCLE_MUTATION_ROUTES = {
    ("POST", "/api/v1/schedules/activation/{employer_id}"),
    ("POST", "/api/v1/schedules/deactivation/{employer_id}"),
    ("POST", "/api/v1/schedules/preview"),
    ("POST", "/api/v1/schedules/preview/rotating"),
    ("POST", "/api/v1/schedules/publish"),
    ("POST", "/api/v1/schedules/publish/rotating"),
    ("POST", "/api/v1/schedules/{employment_id}/{business_date}/break-worked"),
    ("POST", "/api/v1/schedules/{employment_id}/{business_date}/replan/preview"),
    ("POST", "/api/v1/schedules/{employment_id}/{business_date}/replan"),
}

ROUTE_CONFLICT_RESPONSES = LIFECYCLE_GUARDED_ROUTES | SCHEDULE_LIFECYCLE_MUTATION_ROUTES | {
    ("POST", "/api/v1/monitor/alerts/{alert_id}/acknowledge"),
    ("POST", "/api/v1/monitor/alerts/{alert_id}/resolve"),
    ("POST", "/api/v1/employees/{id}/departments"),
    ("POST", "/api/v1/employees/{employee_id}/employments/{employment_id}/terminate"),
}

ROUTE_SERVICE_UNAVAILABLE_RESPONSES = LIFECYCLE_GUARDED_ROUTES | SCHEDULE_LIFECYCLE_MUTATION_ROUTES | {
    ("GET", "/api/v1/monitor/correlations"),
    ("GET", "/api/v1/schedules"),
    ("GET", "/api/v1/schedules/history"),
    ("GET", "/api/v1/schedules/shadow-mismatches"),
    ("POST", "/api/v1/schedules/shadow-mismatches/{mismatch_id}/resolve"),
    ("GET", "/api/v1/schedules/activation-readiness"),
    ("GET", "/api/v1/schedules/evidence/{employment_id}/{business_date}"),
}


SCHEMA_ALIASES = {
    "Employee": "EmployeeRecord",
    "Department": "DepartmentRecord",
    "AuthResponse": "authResponse",
    "ChangePasswordRequest": "changePasswordRequest",
    "RegisterRequest": "registerRequest",
    "ConditionTypeDefinition": "ConditionTypeRecord",
    "CreateIntervalRequest": "ShiftIntervalRequest",
    "CreateWorkingConditionRequest": "CreateEmployeeConditionRequest",
    "EmployeeProfile": "EmployeeProfileRecord",
    "ValidTerminationRow": "ValidTerminationImportRow",
    "InvalidTerminationRow": "InvalidTerminationImportRow",
    "ConfirmTerminationRow": "ConfirmTerminationImportRow",
    "ShiftVersionHistoryItem": "ShiftVersionRecord",
    "CreateOvertimeRequestPayload": "CreateOvertimeRequest",
    "SetCustomFieldValueRequest": "SetCustomFieldValueItem",
    "CreateStorageConfigRequest": "StorageConfigRequest",
    "EmployeeProfile": "EmployeeProfileRecord",
    "ShiftVersionHistoryItem": "ShiftVersionRecord",
    "UpdateWorkingPatternRequest": "CreateWorkingPatternRequest",
    "CreateOvertimeRequestPayload": "CreateOvertimeRequest",
    "ConfirmTerminationRow": "ConfirmTerminationImportRow",
    "ValidTerminationRow": "ValidTerminationImportRow",
    "InvalidTerminationRow": "InvalidTerminationImportRow",
    "TerminationPreviewResponse": "TerminationImportPreviewResponse",
    "LeaveRequest": "LeaveRequestRecord",
    "LeaveType": "LeaveTypeRecord",
    "LeaveTypeConfig": "LeavePolicyConfig",
    "BulkAssignResponse": "BulkAssignPatternResponse",
}


def schema_names() -> list[str]:
    names: set[str] = set()
    for path in (ROOT / "webdashboard/src").rglob("*.ts"):
        if path.name == "api-generated.ts":
            continue
        names.update(SCHEMA_REF.findall(path.read_text(encoding="utf-8")))
    names.update(SCHEMA_ALIASES)
    return sorted(name for name in names if not FORBIDDEN_SCHEMA.search(name))


def go_struct_properties() -> dict[str, dict[str, tuple[str, bool]]]:
    result: dict[str, dict[str, tuple[str, bool]]] = {}
    field_re = re.compile(r'^\s*([A-Za-z][A-Za-z0-9_]*)\s+([^`\s]+(?:\s*\[[^]]+\])?)(?:\s+`json:"([^,\"]+)(?:,[^\"]*)?"`)?')
    for path in (ROOT / "apiserver/internal/api").glob("*.go"):
        lines = path.read_text(encoding="utf-8").splitlines()
        for index, line in enumerate(lines):
            match = re.match(r'^type\s+([A-Za-z][A-Za-z0-9_]*)\s+struct\s*{$', line)
            if not match:
                continue
            name = match.group(1)
            props: dict[str, str] = {}
            for field in lines[index + 1:]:
                if field.strip() == "}":
                    break
                item = field_re.match(field)
                if not item:
                    embedded = re.match(r"^\s*([A-Z][A-Za-z0-9_]*)$", field)
                    if embedded and embedded.group(1) in result:
                        props.update(result[embedded.group(1)])
                    continue
                if not item.group(3) or item.group(3) == "-":
                    embedded = re.match(r"^\s*([A-Z][A-Za-z0-9_]*)", field)
                    if embedded and embedded.group(1) in result:
                        props.update(result[embedded.group(1)])
                    continue
                json_tag = item.group(3)
                go_type = item.group(2).strip()
                props[json_tag.split(",", 1)[0]] = (go_type, "omitempty" not in json_tag and not go_type.startswith("*") and not is_json_type(go_type))
            if props:
                result[name] = props
    return result


def yaml_type(go_type: str) -> str:
    if go_type.startswith("[]"):
        return "array"
    if go_type.startswith("map["):
        return "object"
    if go_type.lstrip("*") in {"int", "int32", "int64", "float32", "float64"}:
        return "number"
    if go_type.lstrip("*") == "bool":
        return "boolean"
    if go_type.lstrip("*") in {"string", "uuid.UUID", "time.Time"}:
        return "string"
    if go_type.lstrip("*") in {"uint", "uint32", "uint64"}:
        return "number"
    if go_type.lstrip("*") == "decimal.Decimal":
        return "number"
    if go_type.lstrip("*") in {"json.RawMessage", "any", "interface{}"}:
        return "object"
    return "string"


def is_json_type(go_type: str) -> bool:
    return go_type.lstrip("*") in {"json.RawMessage", "any", "interface{}"}


def schema_lines(prop: str, go_type: str, known: set[str]) -> list[str]:
    clean = go_type.lstrip("*")
    if is_json_type(go_type):
        return [
            f"        {prop}:",
            "          oneOf:",
            "            - type: string",
            "            - type: number",
            "            - type: boolean",
            "            - type: object",
            "              additionalProperties: true",
            "            - type: array",
            "              items: {}",
            "          nullable: true",
        ]
    if clean.startswith("[]"):
        item = clean[2:]
        if item in known:
            result = [f"        {prop}:", "          type: array", "          items:", f"            $ref: '#/components/schemas/{item}'"]
            if go_type.startswith("*"):
                result.append("          nullable: true")
            return result
        result = [f"        {prop}:", "          type: array", "          items:", f"            type: {yaml_type(item)}"]
        if go_type.startswith("*"):
            result.append("          nullable: true")
        return result
    if clean in known:
        result = [f"        {prop}:", f"          $ref: '#/components/schemas/{clean}'"]
        if go_type.startswith("*"):
            result.append("          nullable: true")
        return result
    if go_type.lstrip("*") == "decimal.Decimal":
        result = [f"        {prop}:", "          type: string"]
    else:
        result = [f"        {prop}:", f"          type: {yaml_type(go_type)}"]
    if go_type.startswith("*"):
        result.append("          nullable: true")
    if is_json_type(go_type):
        result.append("          additionalProperties: true")
    return result


def yaml_quote(value: str) -> str:
    return '"' + value.replace("\\", "\\\\").replace('"', '\\"') + '"'


def route_request_body(method: str, path: str) -> list[str]:
    route = (method, path)
    schema = ROUTE_REQUEST_SCHEMAS.get(route)
    if schema:
        return [
            "      requestBody:",
            "        required: true",
            "        content:",
            "          application/json:",
            "            schema:",
            f"              $ref: '#/components/schemas/{schema}'",
        ]
    if route in ROUTE_MULTIPART_REQUESTS:
        return [
            "      requestBody:",
            "        required: true",
            "        content:",
            "          multipart/form-data:",
            "            schema:",
            "              type: object",
            "              properties:",
            "                file:",
            "                  type: string",
            "                  format: binary",
            "                category:",
            "                  type: string",
            "                owner_type:",
            "                  type: string",
            "                owner_id:",
            "                  type: string",
            "                  format: uuid",
            "              required: [file, category, owner_type]",
        ]
    return []


def generate() -> str:
    routes = route_inventory()
    lines = [
        "openapi: 3.0.3",
        "info:",
        "  title: LeeAttend Go API",
        "  version: 1.0.0",
        "  description: Canonical single-organization API contract owned by Go runtime.",
        "servers:",
        "  - url: /",
        "tags:",
        "  - name: runtime",
        "paths:",
    ]
    grouped: dict[str, list[str]] = {}
    for method, path in routes:
        if path.startswith("/api/"):
            grouped.setdefault(openapi_path(path), []).append(method)
    for path, methods in sorted(grouped.items()):
        lines.append(f"  {yaml_quote(path)}:")
        params = re.findall(r"\{([^{}]+)\}", path)
        if params:
            lines.append("    parameters:")
            for param in params:
                lines.extend(
                    [
                        f"      - name: {param}",
                        "        in: path",
                        "        required: true",
                        "        schema:",
                        "          type: string",
                    ]
                )
        for method in methods:
            request_body = route_request_body(method, path)
            response_status, response_schema = ROUTE_RESPONSE_SCHEMAS.get(
                (method, path), ("200", "JsonObject")
            )
            lines.extend(
                [
                    f"    {method.lower()}:",
                    f"      operationId: {operation_id(method, path)}",
                    "      tags: [runtime]",
                ]
            )
            if request_body:
                lines.extend(request_body)
            lines.extend(
                [
                    "      responses:",
                    f"        '{response_status}':",
                    "          description: Successful response",
                    "          content:",
                    "            application/json:",
                    "              schema:",
                    f"                $ref: '#/components/schemas/{response_schema}'",
                    "        '204':",
                    "          description: Successful response without body",
                    "        '400':",
                    "          $ref: '#/components/responses/BadRequest'",
                    "        '401':",
                    "          $ref: '#/components/responses/Unauthorized'",
                    "        '403':",
                    "          $ref: '#/components/responses/Forbidden'",
                    "        '404':",
                    "          $ref: '#/components/responses/NotFound'",
                ]
            )
            if (method, path) in ROUTE_CONFLICT_RESPONSES:
                lines.extend(
                    [
                        "        '409':",
                        "          $ref: '#/components/responses/Conflict'",
                    ]
                )
            lines.extend(
                [
                    "        '500':",
                    "          $ref: '#/components/responses/InternalError'",
                ]
            )
            if (method, path) in ROUTE_SERVICE_UNAVAILABLE_RESPONSES:
                lines.extend(
                    [
                        "        '503':",
                        "          $ref: '#/components/responses/ServiceUnavailable'",
                    ]
                )
    error_response_content = [
        "      content:",
        "        application/json:",
        "          schema:",
        "            $ref: '#/components/schemas/ErrorEnvelope'",
    ]
    lines.extend(
        [
            "components:",
            "  securitySchemes:",
            "    bearerAuth:",
            "      type: http",
            "      scheme: bearer",
            "      bearerFormat: JWT",
            "  responses:",
            "    BadRequest:",
            "      description: Invalid request",
            *error_response_content,
            "    Unauthorized:",
            "      description: Authentication required",
            *error_response_content,
            "    Forbidden:",
            "      description: Permission denied",
            *error_response_content,
            "    NotFound:",
            "      description: Resource not found",
            *error_response_content,
            "    Conflict:",
            "      description: Resource or schedule lifecycle conflict",
            *error_response_content,
            "    InternalError:",
            "      description: Internal server error",
            *error_response_content,
            "    ServiceUnavailable:",
            "      description: Service or lifecycle schema unavailable",
            *error_response_content,
            "  schemas:",
            "    ErrorEnvelope:",
            "      type: object",
            "      required: [error]",
            "      properties:",
            "        error:",
            "          type: object",
            "          required: [code, message]",
            "          properties:",
            "            code:",
            "              type: string",
            "            message:",
            "              type: string",
            "            retry:",
            "              $ref: '#/components/schemas/JsonObject'",
            "    JsonObject:",
            "      oneOf:",
            "        - type: object",
            "          additionalProperties: true",
            "        - type: string",
            "        - type: number",
            "        - type: boolean",
            "        - type: array",
            "          items: {}",
        ]
    )
    structs = go_struct_properties()
    for name in sorted(set(schema_names()) | set(structs)):

        if name == "JsonObject":
            continue
        if name in EXPLICIT_SCHEMAS:
            lines.extend(EXPLICIT_SCHEMAS[name])
            continue
        alias = SCHEMA_ALIASES.get(name)
        if name in {"Employee", "Department"}:
            lines.append(f"    {name}:")
            lines.append("      allOf:")
            lines.append(f"        - $ref: '#/components/schemas/{alias}'")
            continue
        if alias and alias in structs:
            lines.append(f"    {name}:")
            lines.append("      allOf:")
            lines.append(f"        - $ref: '#/components/schemas/{alias}'")
            continue
        lines.append(f"    {name}:")
        props = structs.get(name, {})
        if not props and name in {"Employee", "Department"}:
            props = structs.get(f"{name}Record", {})
        if not props:
            for candidate in (f"{name}Record", f"{name}Response", f"{name}Result"):
                if candidate in structs:
                    props = structs[candidate]
                    break
        if not props:
            if name.endswith(("Type", "Status", "Category", "Frequency", "Strategy", "Mode", "EntryType")) or name in {"AssigneeType", "FileCategory", "FileOwnerType"}:
                lines.append("      type: string")
            else:
                lines.append("      additionalProperties: true")
            continue
        lines.append("      properties:")
        required: list[str] = []
        known = set(structs)
        for prop, (go_type, is_required) in sorted(props.items()):
            lines.extend(schema_lines(prop, go_type, known))
            if is_required:
                required.append(prop)
        required.extend(REQUIRED_SCHEMA_FIELDS.get(name, ()))
        if required and (not name.endswith("Request") and not name.startswith(("Create", "Update", "Assign", "Bulk")) or name in REQUIRED_SCHEMA_FIELDS):
            lines.extend(["      required:"] + [f"        - {prop}" for prop in sorted(set(required))])
    return "\n".join(lines) + "\n"


if __name__ == "__main__":
    output = ROOT / "apiserver/openapi.yaml"
    output.write_text(generate(), encoding="utf-8")
    print(f"wrote {output}")
    print(f"routes={len(route_inventory())} schemas={len(schema_names())}")
