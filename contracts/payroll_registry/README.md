# payroll_registry Architecture Interface

This document defines the canonical interface and storage model for the
`payroll_registry` contract.

## Entrypoints

| Method | Parameters | Returns | Access Control |
| --- | --- | --- | --- |
| `register_company` | `admin: Address`, `treasury: Address` | `u64` | None |
| `add_employee` | `company_id: u64`, `employee: Address`, `commitment: BytesN<32>` | `()` | `require_auth(admin)` |
| `remove_employee` | `company_id: u64`, `employee: Address` | `()` | `require_auth(admin)` |
| `update_commitment` | `company_id: u64`, `employee: Address`, `new_commitment: BytesN<32>` | `()` | `require_auth(admin)` |
| `set_employee_status` | `company_id: u64`, `employee: Address`, `status: EmployeeStatus` | `()` | `require_auth(admin)` |
| `propose_beneficiary_change` | `company_id: u64`, `current_beneficiary: Address`, `new_beneficiary: Address` | `()` | `require_auth(current_beneficiary)` and `require_auth(admin)` |
| `accept_beneficiary_change` | `company_id: u64`, `current_beneficiary: Address`, `new_beneficiary: Address` | `()` | `require_auth(new_beneficiary)` |
| `cancel_beneficiary_change` | `company_id: u64`, `current_beneficiary: Address`, `admin: Address` | `()` | `require_auth(admin)` |
| `get_pending_beneficiary_change` | `company_id: u64`, `current_beneficiary: Address` | `Option<PendingBeneficiaryChange>` | Public |

## Rust Interface Definitions

The contract interface is defined by the Rust trait:

- `PayrollRegistryTrait`
  - `fn register_company(env: Env, admin: Address, treasury: Address) -> u64`
  - `fn add_employee(env: Env, company_id: u64, employee: Address, commitment: BytesN<32>)`
  - `fn remove_employee(env: Env, company_id: u64, employee: Address)`
  - `fn update_commitment(env: Env, company_id: u64, employee: Address, new_commitment: BytesN<32>)`
  - `fn set_employee_status(env: Env, company_id: u64, employee: Address, status: EmployeeStatus)`
  - `fn propose_beneficiary_change(env: Env, company_id: u64, current_beneficiary: Address, new_beneficiary: Address)`
  - `fn accept_beneficiary_change(env: Env, company_id: u64, current_beneficiary: Address, new_beneficiary: Address)`
  - `fn cancel_beneficiary_change(env: Env, company_id: u64, current_beneficiary: Address, admin: Address)`
  - `fn get_pending_beneficiary_change(env: Env, company_id: u64, current_beneficiary: Address) -> Option<PendingBeneficiaryChange>`

## Storage Types and Keys

Storage is implemented using `DataKey`:

- `DataKey::Company(u64)` maps to `CompanyInfo { admin: Address, treasury: Address }`
- `DataKey::Employee(u64, Address)` maps to `BytesN<32>` (active Poseidon commitment)
- `DataKey::EmpStatus(u64, Address)` maps to `EmployeeStatus`
- `DataKey::PendingBeneficiaryChange(u64, Address)` maps to `PendingBeneficiaryChange`

## Beneficiary Change Verification

The current beneficiary and company admin jointly authorize a proposal. The
proposed beneficiary must then authorize acceptance. Until acceptance, the
existing employee entry remains unchanged and the pending proposal is readable
through `get_pending_beneficiary_change`. Acceptance moves the commitment and
eligibility status to the new address and removes them from the old address.
The admin can cancel a proposal before it is accepted. A destination already
registered for the same company, the current beneficiary itself, or a change
for an unregistered employee is rejected.

The `CompanyInfo` struct definition in Rust:

- `pub struct CompanyInfo { pub admin: Address, pub treasury: Address }`

## Notes

- Company IDs are allocated sequentially and persisted with `DataKey::CompanySequence`.
- Duplicate company registration for the same admin address is rejected with `"Company already registered"`.
- Admin-gated methods load `CompanyInfo` via `DataKey::Company(company_id)` and call
  `info.admin.require_auth()` before mutating employee state.

## Emitted Events

| Event | Trigger |
|-------|---------|
| `CompanyRegistered` | `register_company` |
| `EmployeeAdded` | `add_employee` |
| `EmployeeRemoved` | `remove_employee` |
| `CommitmentUpdated` | `update_commitment` |
| `EmployeeDeactivated` | `set_employee_status(..., EmployeeStatus::Inactive)` |
| `EmployeeReactivated` | `set_employee_status(..., EmployeeStatus::Active)` |
| `EmployeeStatusUpdated` | `set_employee_status(..., EmployeeStatus::Incomplete)` |
| `BeneficiaryChangeProposed` | `propose_beneficiary_change` |
| `BeneficiaryChanged` | `accept_beneficiary_change` |
| `BeneficiaryChangeCancelled` | `cancel_beneficiary_change` |
