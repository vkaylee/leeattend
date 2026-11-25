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
    ("POST", "/api/v1/files/presigned"): "CreateUploadUrlRequest",
}

ROUTE_MULTIPART_REQUESTS = {
    ("POST", "/api/v1/files/upload"),
}

ROUTE_RESPONSE_SCHEMAS = {
    ("POST", "/api/v1/employees/match"): ("200", "MatchEmployeeResponse"),
    ("POST", "/api/v1/files/presigned"): ("200", "CreateUploadUrlResponse"),
    ("POST", "/api/v1/files/upload"): ("201", "FileRecord"),
}

REQUIRED_SCHEMA_FIELDS = {
    "CreateUploadUrlRequest": ("category", "file_name", "file_size", "mime_type", "owner_type"),
    "CreateUploadUrlResponse": ("file", "upload_url"),
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
                    "        '500':",
                    "          $ref: '#/components/responses/InternalError'",
                ]
            )
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
            "    Unauthorized:",
            "      description: Authentication required",
            "    Forbidden:",
            "      description: Permission denied",
            "    NotFound:",
            "      description: Resource not found",
            "    InternalError:",
            "      description: Internal server error",
            "  schemas:",
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
