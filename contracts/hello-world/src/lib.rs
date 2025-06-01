#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Map, Symbol, Vec};

#[derive(Clone)]
#[contracttype]
pub struct SplitInfo {
    member: Address,
    share: i128, // Share in basis points (e.g., 5000 for 50%)
}

#[derive(Clone)]
#[contracttype]
pub struct Group {
    members: Vec<Address>,
    total_amount: i128,
    member_shares: Map<Address, i128>,
}

#[derive(Clone)]
#[contracttype]
pub struct Expense {
    payer: Address,
    amount: i128,
    description: Symbol,
    split_info: Vec<SplitInfo>,
    timestamp: u64,
}

#[contracttype]
pub enum DataKey {
    GroupCounter,
    Group(u32),
    MemberGroups(Address),
    GroupExpenses(u32),
}

#[contract]
pub struct SplitPayment;

#[contractimpl]
impl SplitPayment {
    pub fn create_group(env: Env, members: Vec<Address>) -> u32 {
        // Validate members list is not empty
        if members.len() == 0 {
            panic!("Group must have at least one member");
        }

        // Initialize group counter if not exists
        if !env.storage().instance().has(&0) {
            env.storage().instance().set(&0, &0u32);
        }

        // Generate new group ID
        let group_id = env.storage().instance().get(&0).unwrap_or(0) + 1;

        // Create new group
        let group = Group {
            members: members.clone(),
            total_amount: 0,
            member_shares: Map::new(&env),
        };

        // Store group
        env.storage().instance().set(&group_id, &group);
        env.storage().instance().set(&0, &group_id);

        // Store group references for each member
        for member in members.iter() {
            let mut member_groups: Vec<u32> = env
                .storage()
                .instance()
                .get(&DataKey::MemberGroups(member.clone()))
                .unwrap_or(Vec::new(&env));
            member_groups.push_back(group_id);
            env.storage()
                .instance()
                .set(&DataKey::MemberGroups(member), &member_groups);
        }

        group_id
    }

    pub fn add_member(env: Env, group_id: u32, new_member: Address) {
        let mut group: Group = env.storage().instance().get(&group_id).unwrap();

        // Check if member already exists
        if group.members.contains(&new_member) {
            panic!("Member already exists in group");
        }

        // Add member to group
        group.members.push_back(new_member.clone());

        // Update group in storage
        env.storage().instance().set(&group_id, &group);

        // Update member's group references
        let mut member_groups: Vec<u32> = env
            .storage()
            .instance()
            .get(&DataKey::MemberGroups(new_member.clone()))
            .unwrap_or(Vec::new(&env));
        member_groups.push_back(group_id);
        env.storage()
            .instance()
            .set(&DataKey::MemberGroups(new_member), &member_groups);
    }

    pub fn remove_member(env: Env, group_id: u32, member: Address) {
        let mut group: Group = env.storage().instance().get(&group_id).unwrap();

        // Verify member exists
        if !group.members.contains(&member) {
            panic!("Member not found in group");
        }

        // Check if member has outstanding balance
        let balance = group.member_shares.get(member.clone()).unwrap_or(0);
        if balance != 0 {
            panic!("Member has non-zero balance");
        }

        // Remove member from group
        let idx = group.members.first_index_of(&member).unwrap();
        group.members.remove(idx);

        // Update group in storage
        env.storage().instance().set(&group_id, &group);

        // Update member's group references
        let member_groups_key = DataKey::MemberGroups(member.clone());
        if let Some(member_groups) = env.storage().instance().get(&member_groups_key) {
            let mut updated_groups: Vec<u32> = member_groups;
            if let Some(idx) = updated_groups.first_index_of(&group_id) {
                updated_groups.remove(idx);
                env.storage()
                    .instance()
                    .set(&DataKey::MemberGroups(member), &updated_groups);
            }
        }
    }

    pub fn get_group_members(env: Env, group_id: u32) -> Vec<Address> {
        let group: Group = env.storage().instance().get(&group_id).unwrap();
        group.members
    }

    pub fn add_expense(
        env: Env,
        group_id: u32,
        payer: Address,
        amount: i128,
        description: Symbol,
        split_members: Vec<SplitInfo>,
    ) -> u32 {
        let mut group: Group = env.storage().instance().get(&group_id).unwrap();

        // Verify payer is a member
        if !group.members.contains(&payer) {
            panic!("Payer is not a group member");
        }

        // Verify amount
        if amount <= 0 {
            panic!("Amount must be positive");
        }

        // Verify split members
        for split in split_members.iter() {
            if !group.members.contains(&split.member) {
                panic!("Split member is not in group");
            }
        }

        // Verify total split equals 100%
        let total_split: i128 = split_members.iter().map(|s| s.share).sum();
        if total_split != 10000 {
            // 10000 basis points = 100%
            panic!("Split shares must total 100%");
        }

        // Create expense record
        let expense = Expense {
            payer: payer.clone(),
            amount,
            description,
            split_info: split_members.clone(),
            timestamp: env.ledger().timestamp(),
        };

        // Store expense
        let expenses_key = DataKey::GroupExpenses(group_id);
        let mut expenses: Vec<Expense> = env
            .storage()
            .instance()
            .get(&expenses_key)
            .unwrap_or(Vec::new(&env));
        expenses.push_back(expense);
        env.storage().instance().set(&expenses_key, &expenses);

        // Update member shares
        for split in split_members.iter() {
            let member = split.member.clone();
            let current = group.member_shares.get(member.clone()).unwrap_or(0);
            let member_share = (amount * split.share) / 10000;

            if member == payer {
                group
                    .member_shares
                    .set(member.clone(), current + amount - member_share);
            } else {
                group
                    .member_shares
                    .set(member.clone(), current - member_share);
            }
        }

        // Update total amount
        group.total_amount += amount;
        env.storage().instance().set(&group_id, &group);

        group_id
    }

    pub fn remove_expense(env: Env, group_id: u32, expense_index: u32, authorized_by: Address) {
        let mut group: Group = env.storage().instance().get(&group_id).unwrap();

        // Get expenses
        let expenses_key = DataKey::GroupExpenses(group_id);
        let mut expenses: Vec<Expense> = env
            .storage()
            .instance()
            .get(&expenses_key)
            .unwrap_or(Vec::new(&env));

        // Check if expense index is valid
        if expense_index as u32 >= expenses.len() {
            panic!("Invalid expense index");
        }

        // Get the expense to remove
        let expense = expenses.get(expense_index).unwrap();

        // Verify the person removing is the original payer
        if expense.payer != authorized_by {
            panic!("Only the original payer can remove an expense");
        }

        // Reset balances by inverting the original split calculations
        for split in expense.split_info.iter() {
            let member = split.member.clone();
            let current = group.member_shares.get(member.clone()).unwrap_or(0);
            let member_share = (expense.amount * split.share) / 10000;

            if member == expense.payer {
                // For payer: subtract the full amount and subtract their share
                // Opposite of: current + amount - member_share
                group
                    .member_shares
                    .set(member.clone(), current - (expense.amount - member_share));
            } else {
                // For others: add their share back
                // Opposite of: current - member_share
                group
                    .member_shares
                    .set(member.clone(), current + member_share);
            }
        }

        // Update total amount
        group.total_amount -= expense.amount;

        // Remove the expense from the list
        expenses.remove(expense_index);

        // Update storage
        env.storage().instance().set(&group_id, &group);
        env.storage().instance().set(&expenses_key, &expenses);
    }

    pub fn settle_debt(env: Env, group_id: u32, from: Address, to: Address, amount: i128) {
        let mut group: Group = env.storage().instance().get(&group_id).unwrap();

        // Verify both addresses are members
        if !group.members.contains(&from) {
            panic!("From address is not a group member");
        }
        if !group.members.contains(&to) {
            panic!("To address is not a group member");
        }

        // Verify amount
        if amount <= 0 {
            panic!("Amount must be positive");
        }

        let from_share = group.member_shares.get(from.clone()).unwrap_or(0);
        let to_share = group.member_shares.get(to.clone()).unwrap_or(0);

        // Check if the person has debt (negative balance)
        if from_share >= 0 {
            panic!("From address does not owe any money");
        }

        // Check if trying to settle more than what is owed
        // from_share is negative, so we want amount to be less than or equal to its absolute value
        if amount > (-from_share) {
            panic!("Cannot settle more than what is owed");
        }

        // Update balances
        group.member_shares.set(from.clone(), from_share + amount);
        group.member_shares.set(to.clone(), to_share - amount);

        env.storage().instance().set(&group_id, &group);
    }

    // Get member balance
    pub fn get_member_balance(env: Env, group_id: u32, member: Address) -> i128 {
        let group: Group = env.storage().instance().get(&group_id).unwrap();
        if !group.members.contains(&member) {
            panic!("Address is not a group member");
        }
        group.member_shares.get(member).unwrap_or(0)
    }

    // Get group expenses
    pub fn get_group_expenses(env: Env, group_id: u32) -> Vec<Expense> {
        env.storage()
            .instance()
            .get(&DataKey::GroupExpenses(group_id))
            .unwrap_or(Vec::new(&env))
    }
}
