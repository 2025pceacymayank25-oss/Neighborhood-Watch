#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    Address, Env, Map, String, Symbol, Vec,
};

// ─── Storage Keys ───────────────────────────────────────────────────────────

const INCIDENTS: Symbol = symbol_short!("INCIDENTS");
const STAKES: Symbol = symbol_short!("STAKES");
const INCIDENT_COUNT: Symbol = symbol_short!("INC_CNT");
const MIN_STAKE: Symbol = symbol_short!("MIN_STAKE");
const ADMIN: Symbol = symbol_short!("ADMIN");

// ─── Types ───────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum IncidentStatus {
    Pending,
    Verified,
    Disputed,
    Resolved,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Incident {
    pub id: u64,
    pub reporter: Address,
    pub description: String,
    pub location: String,
    pub severity: u32,        // 1 (low) → 5 (critical)
    pub stake_amount: i128,
    pub status: IncidentStatus,
    pub verifier_count: u32,
    pub dispute_count: u32,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct StakeInfo {
    pub amount: i128,
    pub reputation: u32,      // increases with successful verifications
    pub reports_filed: u32,
    pub verifications_done: u32,
}

// ─── Contract ────────────────────────────────────────────────────────────────

#[contract]
pub struct SafetyNetContract;

#[contractimpl]
impl SafetyNetContract {

    // ── Initialization ───────────────────────────────────────────────────────

    /// Initialize the contract with admin and minimum stake requirement.
    pub fn initialize(env: Env, admin: Address, min_stake: i128) {
        if env.storage().instance().has(&ADMIN) {
            panic!("Already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&ADMIN, &admin);
        env.storage().instance().set(&MIN_STAKE, &min_stake);
        env.storage().instance().set(&INCIDENT_COUNT, &0u64);
    }

    // ── Staking ──────────────────────────────────────────────────────────────

    /// Resident deposits stake to participate in the network.
    pub fn deposit_stake(env: Env, resident: Address, amount: i128) {
        resident.require_auth();

        let min_stake: i128 = env.storage().instance().get(&MIN_STAKE).unwrap();
        if amount < min_stake {
            panic!("Stake below minimum required");
        }

        let mut stakes: Map<Address, StakeInfo> = env
            .storage()
            .instance()
            .get(&STAKES)
            .unwrap_or(Map::new(&env));

        let existing = stakes.get(resident.clone()).unwrap_or(StakeInfo {
            amount: 0,
            reputation: 100,
            reports_filed: 0,
            verifications_done: 0,
        });

        stakes.set(
            resident,
            StakeInfo {
                amount: existing.amount + amount,
                ..existing
            },
        );

        env.storage().instance().set(&STAKES, &stakes);
    }

    /// Resident withdraws their stake (only if not locked in active reports).
    pub fn withdraw_stake(env: Env, resident: Address, amount: i128) {
        resident.require_auth();

        let mut stakes: Map<Address, StakeInfo> = env
            .storage()
            .instance()
            .get(&STAKES)
            .unwrap_or(Map::new(&env));

        let info = stakes.get(resident.clone()).expect("No stake found");
        if info.amount < amount {
            panic!("Insufficient stake balance");
        }

        stakes.set(
            resident,
            StakeInfo {
                amount: info.amount - amount,
                ..info
            },
        );

        env.storage().instance().set(&STAKES, &stakes);
    }

    // ── Incident Reporting ───────────────────────────────────────────────────

    /// File a new safety incident report. Reporter's stake is partially locked.
    pub fn report_incident(
        env: Env,
        reporter: Address,
        description: String,
        location: String,
        severity: u32,
        stake_amount: i128,
    ) -> u64 {
        reporter.require_auth();

        if severity < 1 || severity > 5 {
            panic!("Severity must be between 1 and 5");
        }

        // Validate reporter has sufficient stake
        let mut stakes: Map<Address, StakeInfo> = env
            .storage()
            .instance()
            .get(&STAKES)
            .unwrap_or(Map::new(&env));

        let min_stake: i128 = env.storage().instance().get(&MIN_STAKE).unwrap();
        let mut reporter_stake = stakes.get(reporter.clone()).expect("Must stake before reporting");

        if reporter_stake.amount < min_stake {
            panic!("Insufficient stake to report incident");
        }

        // Lock stake for this incident
        reporter_stake.amount -= stake_amount;
        reporter_stake.reports_filed += 1;
        stakes.set(reporter.clone(), reporter_stake);
        env.storage().instance().set(&STAKES, &stakes);

        // Create incident
        let mut count: u64 = env.storage().instance().get(&INCIDENT_COUNT).unwrap();
        count += 1;

        let incident = Incident {
            id: count,
            reporter: reporter.clone(),
            description,
            location,
            severity,
            stake_amount,
            status: IncidentStatus::Pending,
            verifier_count: 0,
            dispute_count: 0,
            timestamp: env.ledger().timestamp(),
        };

        let mut incidents: Map<u64, Incident> = env
            .storage()
            .instance()
            .get(&INCIDENTS)
            .unwrap_or(Map::new(&env));

        incidents.set(count, incident);
        env.storage().instance().set(&INCIDENTS, &incidents);
        env.storage().instance().set(&INCIDENT_COUNT, &count);

        env.events().publish(
            (Symbol::new(&env, "incident_reported"), reporter),
            count,
        );

        count
    }

    // ── Verification & Disputes ──────────────────────────────────────────────

    /// A staked resident verifies (confirms) an incident as legitimate.
    pub fn verify_incident(env: Env, verifier: Address, incident_id: u64) {
        verifier.require_auth();

        // Verifier must have stake
        let mut stakes: Map<Address, StakeInfo> = env
            .storage()
            .instance()
            .get(&STAKES)
            .unwrap_or(Map::new(&env));

        let mut verifier_stake = stakes.get(verifier.clone()).expect("Must stake to verify");
        let min_stake: i128 = env.storage().instance().get(&MIN_STAKE).unwrap();
        if verifier_stake.amount < min_stake {
            panic!("Insufficient stake to verify");
        }

        let mut incidents: Map<u64, Incident> = env
            .storage()
            .instance()
            .get(&INCIDENTS)
            .unwrap_or(Map::new(&env));

        let mut incident = incidents.get(incident_id).expect("Incident not found");

        if incident.status != IncidentStatus::Pending {
            panic!("Incident is not in Pending state");
        }
        if incident.reporter == verifier {
            panic!("Reporter cannot verify their own incident");
        }

        incident.verifier_count += 1;

        // Auto-verify after 3 independent verifications
        if incident.verifier_count >= 3 {
            incident.status = IncidentStatus::Verified;

            // Reward reporter's reputation
            let mut reporter_stake = stakes
                .get(incident.reporter.clone())
                .unwrap_or(StakeInfo { amount: 0, reputation: 100, reports_filed: 0, verifications_done: 0 });
            reporter_stake.reputation += 10;
            reporter_stake.amount += incident.stake_amount; // return locked stake
            stakes.set(incident.reporter.clone(), reporter_stake);

            env.events().publish(
                (Symbol::new(&env, "incident_verified"), incident.reporter.clone()),
                incident_id,
            );
        }

        // Reward verifier reputation
        verifier_stake.reputation += 5;
        verifier_stake.verifications_done += 1;
        stakes.set(verifier, verifier_stake);

        incidents.set(incident_id, incident);
        env.storage().instance().set(&INCIDENTS, &incidents);
        env.storage().instance().set(&STAKES, &stakes);
    }

    /// A staked resident disputes an incident as false or inaccurate.
    pub fn dispute_incident(env: Env, disputer: Address, incident_id: u64) {
        disputer.require_auth();

        let mut stakes: Map<Address, StakeInfo> = env
            .storage()
            .instance()
            .get(&STAKES)
            .unwrap_or(Map::new(&env));

        let disputer_stake = stakes.get(disputer.clone()).expect("Must stake to dispute");
        let min_stake: i128 = env.storage().instance().get(&MIN_STAKE).unwrap();
        if disputer_stake.amount < min_stake {
            panic!("Insufficient stake to dispute");
        }

        let mut incidents: Map<u64, Incident> = env
            .storage()
            .instance()
            .get(&INCIDENTS)
            .unwrap_or(Map::new(&env));

        let mut incident = incidents.get(incident_id).expect("Incident not found");
        if incident.status != IncidentStatus::Pending {
            panic!("Only pending incidents can be disputed");
        }

        incident.dispute_count += 1;

        // Auto-dispute threshold: more disputes than verifiers
        if incident.dispute_count > incident.verifier_count && incident.dispute_count >= 2 {
            incident.status = IncidentStatus::Disputed;

            // Slash reporter's stake (partial penalty)
            let mut reporter_stake = stakes
                .get(incident.reporter.clone())
                .unwrap_or(StakeInfo { amount: 0, reputation: 100, reports_filed: 0, verifications_done: 0 });
            let slash_amount = incident.stake_amount / 2;
            reporter_stake.reputation = reporter_stake.reputation.saturating_sub(20);
            // Slashed amount stays in contract as community pool (simplified)
            reporter_stake.amount += incident.stake_amount - slash_amount;
            stakes.set(incident.reporter.clone(), reporter_stake);

            env.events().publish(
                (Symbol::new(&env, "incident_disputed"), disputer.clone()),
                incident_id,
            );
        }

        stakes.set(disputer, disputer_stake);
        incidents.set(incident_id, incident);
        env.storage().instance().set(&INCIDENTS, &incidents);
        env.storage().instance().set(&STAKES, &stakes);
    }

    // ── Admin ────────────────────────────────────────────────────────────────

    /// Admin can mark a verified incident as resolved.
    pub fn resolve_incident(env: Env, incident_id: u64) {
        let admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        admin.require_auth();

        let mut incidents: Map<u64, Incident> = env
            .storage()
            .instance()
            .get(&INCIDENTS)
            .unwrap_or(Map::new(&env));

        let mut incident = incidents.get(incident_id).expect("Incident not found");
        if incident.status != IncidentStatus::Verified {
            panic!("Only verified incidents can be resolved");
        }
        incident.status = IncidentStatus::Resolved;
        incidents.set(incident_id, incident);
        env.storage().instance().set(&INCIDENTS, &incidents);
    }

    /// Admin updates minimum stake threshold.
    pub fn set_min_stake(env: Env, new_min: i128) {
        let admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        admin.require_auth();
        env.storage().instance().set(&MIN_STAKE, &new_min);
    }

    // ── Queries ──────────────────────────────────────────────────────────────

    pub fn get_incident(env: Env, incident_id: u64) -> Incident {
        let incidents: Map<u64, Incident> = env
            .storage()
            .instance()
            .get(&INCIDENTS)
            .unwrap_or(Map::new(&env));
        incidents.get(incident_id).expect("Incident not found")
    }

    pub fn get_stake_info(env: Env, resident: Address) -> StakeInfo {
        let stakes: Map<Address, StakeInfo> = env
            .storage()
            .instance()
            .get(&STAKES)
            .unwrap_or(Map::new(&env));
        stakes.get(resident).unwrap_or(StakeInfo {
            amount: 0,
            reputation: 0,
            reports_filed: 0,
            verifications_done: 0,
        })
    }

    pub fn get_incident_count(env: Env) -> u64 {
        env.storage().instance().get(&INCIDENT_COUNT).unwrap_or(0)
    }

    pub fn get_min_stake(env: Env) -> i128 {
        env.storage().instance().get(&MIN_STAKE).unwrap_or(0)
    }

    pub fn get_pending_incidents(env: Env) -> Vec<u64> {
        let incidents: Map<u64, Incident> = env
            .storage()
            .instance()
            .get(&INCIDENTS)
            .unwrap_or(Map::new(&env));
        let count: u64 = env.storage().instance().get(&INCIDENT_COUNT).unwrap_or(0);
        let mut pending = Vec::new(&env);
        for i in 1..=count {
            if let Some(inc) = incidents.get(i) {
                if inc.status == IncidentStatus::Pending {
                    pending.push_back(i);
                }
            }
        }
        pending
    }
}