use payroll::{PayoutConfirmationRetryDecision, Payroll, PayrollClient};
use soroban_sdk::testutils::Ledger as _;
use soroban_sdk::Env;

fn setup(env: &Env) -> PayrollClient<'_> {
    let contract_id = env.register_contract(None, Payroll);
    PayrollClient::new(env, &contract_id)
}

#[test]
fn test_confirmation_retry_waits_then_becomes_allowed() {
    let env = Env::default();
    let client = setup(&env);
    env.ledger().with_mut(|ledger| ledger.timestamp = 129);

    let waiting = client.get_payout_confirmation_retry_decision(&0, &100);
    assert_eq!(waiting.retries_used, 0);
    assert_eq!(waiting.max_retries, 5);
    assert_eq!(waiting.retry_after, Some(130));
    assert!(!waiting.retry_allowed);
    assert!(!waiting.exhausted);

    env.ledger().with_mut(|ledger| ledger.timestamp = 130);
    let due = client.get_payout_confirmation_retry_decision(&0, &100);
    assert!(due.retry_allowed);
}

#[test]
fn test_confirmation_retry_uses_exponential_backoff() {
    let env = Env::default();
    let client = setup(&env);
    env.ledger().with_mut(|ledger| ledger.timestamp = 220);

    let decision = client.get_payout_confirmation_retry_decision(&2, &100);
    assert_eq!(
        decision,
        PayoutConfirmationRetryDecision {
            retries_used: 2,
            max_retries: 5,
            retry_after: Some(220),
            retry_allowed: true,
            exhausted: false,
        }
    );
}

#[test]
fn test_confirmation_retry_limit_is_enforced() {
    let env = Env::default();
    let client = setup(&env);
    env.ledger().with_mut(|ledger| ledger.timestamp = 1_000);

    let decision = client.get_payout_confirmation_retry_decision(&5, &100);
    assert_eq!(decision.retry_after, None);
    assert!(!decision.retry_allowed);
    assert!(decision.exhausted);
}

#[test]
fn test_confirmation_retry_rejects_future_last_attempt_timestamp() {
    let env = Env::default();
    let client = setup(&env);
    env.ledger().with_mut(|ledger| ledger.timestamp = 100);

    let result = client.try_get_payout_confirmation_retry_decision(&0, &101);
    assert!(result.is_err());
}
