use tari_engine_types::instruction::Instruction;
use tari_template_lib::{
    args,
    prelude::{ComponentAddress, NonFungibleAddress, TemplateAddress},
};
use utils::TemplateTest;

mod utils;

// struct definition inside the "lp_position" template
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct Position {
    pub provided_token: String,
    pub provided_token_balance: u64,
    pub requested_token: String,
    pub requested_token_balance: u64,
}

#[derive(Clone)]
struct User {
    owner_token: NonFungibleAddress,
    position_component: ComponentAddress,
}

struct LpTest {
    template_test: TemplateTest,
    lp_index_component: ComponentAddress,
    users: Vec<User>,
}

fn setup() -> LpTest {
    // get template addresses
    let root = env!("CARGO_MANIFEST_DIR");
    let lp_index_template_path = format!("{}/templates/lp_index", root);
    let lp_position_template_path = format!("{}/templates/lp_position", root);
    let mut template_test =
        TemplateTest::new(vec![lp_index_template_path, lp_position_template_path]);
    let lp_index_template = template_test.get_template_address("LiquidityProviderIndex");
    let lp_position_template = template_test.get_template_address("LiquidityProviderPosition");

    // create the component for the lp index
    let lp_index_component =
        create_lp_index_component(&mut template_test, lp_index_template, lp_position_template);

    LpTest {
        template_test,
        lp_index_component,
        users: vec![],
    }
}

fn create_lp_index_component(
    test: &mut TemplateTest,
    lp_index_template: TemplateAddress,
    lp_position_template: TemplateAddress,
) -> ComponentAddress {
    let result = test
        .execute_and_commit(
            vec![Instruction::CallFunction {
                template_address: lp_index_template,
                function: "new".to_string(),
                args: args![lp_position_template],
            }],
            // Sender proof needed to withdraw
            vec![],
        )
        .unwrap();
    result.finalize.execution_results[0].decode().unwrap()
}

fn register_lp(test: &mut LpTest, network_address: String) {
    let (_, owner_token, _) = test.template_test.create_owned_account();
    let result = test
        .template_test
        .execute_and_commit(
            vec![Instruction::CallMethod {
                component_address: test.lp_index_component,
                method: "register".to_string(),
                args: args![owner_token, network_address],
            }],
            // Sender proof needed to withdraw
            vec![],
        )
        .unwrap();
    let position_component: ComponentAddress =
        result.finalize.execution_results[0].decode().unwrap();

    let user = User {
        owner_token,
        position_component,
    };

    test.users.push(user);
}

fn add_position(test: &mut LpTest, user_index: usize, position: Position) {
    let user = test.users[user_index].clone();
    test.template_test
        .execute_and_commit(
            vec![Instruction::CallMethod {
                component_address: user.position_component,
                method: "add_position".to_string(),
                args: args![position],
            }],
            vec![user.owner_token],
        )
        .unwrap();
}

fn remove_position(test: &mut LpTest, user_index: usize, position_index: usize) {
    let user = test.users[user_index].clone();
    test.template_test
        .execute_and_commit(
            vec![Instruction::CallMethod {
                component_address: user.position_component,
                method: "remove_position".to_string(),
                args: args![position_index],
            }],
            vec![user.owner_token],
        )
        .unwrap();
}

fn get_positions(test: &mut LpTest, user_index: usize) -> Vec<Position> {
    let user = test.users[user_index].clone();
    let result = test
        .template_test
        .execute_and_commit(
            vec![Instruction::CallMethod {
                component_address: user.position_component,
                method: "get_positions".to_string(),
                args: args![],
            }],
            vec![],
        )
        .unwrap();
    result.finalize.execution_results[0].decode().unwrap()
}

fn replace_positions(test: &mut LpTest, user_index: usize, positions: Vec<Position>) {
    let user = test.users[user_index].clone();
    test.template_test
        .execute_and_commit(
            vec![Instruction::CallMethod {
                component_address: user.position_component,
                method: "replace_positions".to_string(),
                args: args![positions],
            }],
            vec![user.owner_token],
        )
        .unwrap();
}

#[test]
fn it_allows_to_add_and_remove_positions() {
    // setup a index with one lp registered
    let mut test = setup();
    register_lp(&mut test, "http://alice".to_owned());
    let alice_index = 0;

    // initial state for a user is empty
    let positions = get_positions(&mut test, alice_index);
    assert!(positions.is_empty());

    // adding a new position
    add_position(
        &mut test,
        alice_index,
        Position {
            provided_token: "tari".to_string(),
            provided_token_balance: 100000,
            requested_token: "eth_wei".to_string(),
            requested_token_balance: 20000,
        },
    );
    let positions = get_positions(&mut test, alice_index);
    assert_eq!(positions.len(), 1);

    // removing a position
    remove_position(&mut test, alice_index, 0);
    let positions = get_positions(&mut test, alice_index);
    assert!(positions.is_empty());
}

#[test]
fn it_allows_to_replace_positions() {
    // setup a index with one lp registered
    let mut test = setup();
    register_lp(&mut test, "http://alice".to_owned());
    let alice_index = 0;

    // initial state for a user is empty
    let positions = get_positions(&mut test, alice_index);
    assert!(positions.is_empty());

    // adding a new position
    add_position(
        &mut test,
        alice_index,
        Position {
            provided_token: "tari".to_string(),
            provided_token_balance: 100000,
            requested_token: "eth_wei".to_string(),
            requested_token_balance: 20000,
        },
    );

    // replace positions
    let new_positions = vec![
        Position {
            provided_token: "tari".to_string(),
            provided_token_balance: 100001,
            requested_token: "eth_wei".to_string(),
            requested_token_balance: 20001,
        },
        Position {
            provided_token: "tari".to_string(),
            provided_token_balance: 100002,
            requested_token: "eth_wei".to_string(),
            requested_token_balance: 20002,
        },
    ];
    replace_positions(&mut test, alice_index, new_positions.clone());
    let positions = get_positions(&mut test, alice_index);
    assert_eq!(new_positions, positions);
}

#[test]
fn it_does_not_allow_updates_from_undesignated_users() {
    // setup a index with two providers
    let mut test = setup();
    register_lp(&mut test, "http://alice".to_owned());
    register_lp(&mut test, "http://bob".to_owned());
    let alice = test.users[0].clone();
    let bob = test.users[1].clone();

    // try to add a position for Alice using Bob's credentials
    let position = Position {
        provided_token: "tari".to_string(),
        provided_token_balance: 100000,
        requested_token: "eth_wei".to_string(),
        requested_token_balance: 20000,
    };
    let err = test
        .template_test
        .execute_and_commit(
            vec![Instruction::CallMethod {
                component_address: alice.position_component,
                method: "add_position".to_string(),
                args: args![position],
            }],
            vec![bob.owner_token],
        )
        .unwrap_err();

    // Bob should not be able to update Alice's positions
    assert!(err.to_string().contains("Access Denied"));
}
