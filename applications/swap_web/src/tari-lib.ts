import { TariConnection } from 'tari-connector/src/index';
import * as cbor from 'cbor-web'

let lp_index_component = import.meta.env.VITE_TARI_LP_INDEX;

async function get_best_match(tari: TariConnection, provided_token, provided_token_balance, requested_token) {
    let all_provider_positions = await get_all_provider_positions(tari);
    let best_match;

    all_provider_positions.forEach(function (provider) {
        // get the position (if any) that this provider offers that match our request
        // we assume a provider will publish at most only one offer for a particular pair of tokens
        let position = provider.positions.find((p) =>
            p.requested_token == provided_token &&
            p.requested_token_balance >= provided_token_balance &&
            p.provided_token == requested_token
        );

        // skip this provider if it has no matches with our request
        if(!position) {
            return;
        }

        // expected tokens to receive
        let ratio = position.provided_token_balance / position.requested_token_balance;
        let expected_balance = provided_token_balance * ratio;
        console.log({position, ratio, expected_balance});

        // data for holding this provider's best possible match for our request
        const provider_best_match = {
            network_address: provider.network_address,
            public_key: provider.public_key,
            position,
            expected_balance,
        };
        
        // update the best match so far if this provider has the best offer
        if(!best_match || provider_best_match.expected_balance > best_match.expected_balance) {
            best_match = provider_best_match;
        }
    });

    return best_match;
}

async function get_all_provider_positions(tari: TariConnection) {
    let submit_resp = await tari.sendMessage("transactions.submit", tari.token,
    /*signing_key_index: */ null,
    /*fee_instructions":*/[
    ],
    /*instructions":*/[
        {
            "CallMethod": {
                "component_address": lp_index_component,
                "method": "get_all_provider_positions",
                "args": []
            }
        },
    ],
    /*inputs":*/[{ "address": lp_index_component }],
    /*override_inputs":*/ false,
    /*new_outputs":*/ 0,
    /*specific_non_fungible_outputs":*/[],
    /*new_resources":*/[],
    /*new_non_fungible_outputs":*/[],
    /*new_non_fungible_index_outputs":*/[],
    /*is_dry_run":*/ true,
    /*proof_ids":*/[]
    );

    return submit_resp.result.finalize.execution_results[0].json;
};

async function withdraw(tari: TariConnection, contract, preimage) {
    console.log("tari-lib withdraw");
    let res = await tari.sendMessage("accounts.get_default", tari.token);
    console.log({res});
    let key_index = res.account.key_index;
    let account = res.account.address.Component;
    //let preimage_cbor = [152,32, ...preimage];
    console.log({preimage});
    let preimage_cbor = encode_cbor(preimage);
    console.log({preimage_cbor});

    let submit_resp = await tari.sendMessage("transactions.submit", tari.token,
    /*signing_key_index: */ null,
    /*fee_instructions":*/[
    ],
    /*instructions":*/[
        {
            "CallMethod": {
                "component_address": contract,
                "method": "withdraw",
                "args": [{"Literal": preimage_cbor}]
            }
        },
        {
            "PutLastInstructionOutputOnWorkspace": {
                "key": [98, 95, 98, 117, 99, 107, 101, 116]
            }
        },
        {
            "CallMethod": {
                "component_address": account,
                "method": "deposit",
                "args": [{ "Workspace": [98, 95, 98, 117, 99, 107, 101, 116] }]
            }
        },
    ],
    /*inputs":*/[{ "address": contract }, { "address": account }],
    /*override_inputs":*/ false,
    /*new_outputs":*/ 0,
    /*specific_non_fungible_outputs":*/[],
    /*new_resources":*/[],
    /*new_non_fungible_outputs":*/[],
    /*new_non_fungible_index_outputs":*/[],
    /*is_dry_run":*/ false,
    /*proof_ids":*/[]
    );
    console.log({submit_resp});

    //let wait_resp = await tari.sendMessage("transactions.wait_result", tari.token, submit_resp.hash, 15);
    //console.log({ wait_resp });
};

function encode_cbor(value) {
    const buffer = cbor.Encoder.encode(value);
    let value_cbor_bytes = [...buffer.values()];
    return value_cbor_bytes;
}

export { get_best_match, get_all_provider_positions, withdraw };