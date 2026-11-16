# ADR 001: Retired Multi-Organization SaaS Architecture

## Status
Superseded by [ADR 0005](0005-single-organization-schema-cutover.md). This record preserves prior architectural rationale; current LeeAttend uses one standalone organization and one PostgreSQL database.

## Historical context
LeeAttend previously evaluated separate databases and request-selected organization scope for SaaS isolation. That design is no longer active. Current authorization uses authenticated identity, explicit permissions, ownership, department boundaries, and fail-closed checks; it does not use a client-selected organization selector.

## Historical trade-offs
The retired design considered database-per-customer isolation, dynamic connection routing, per-customer migration orchestration, and provisioning flows. These concerns no longer define runtime behavior. Git history preserves original evidence; current architecture and operational procedures live in ADR 0005 and its linked migration plan.
