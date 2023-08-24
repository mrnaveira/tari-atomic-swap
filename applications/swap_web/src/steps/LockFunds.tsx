import * as React from 'react';
import Typography from '@mui/material/Typography';
import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import SyncAltIcon from '@mui/icons-material/SyncAlt';
import List from '@mui/material/List';
import ListItem from '@mui/material/ListItem';
import ListItemText from '@mui/material/ListItemText';
import Stack from '@mui/material/Stack';
import * as  CryptoJS from 'crypto-js';
import sha256 from 'crypto-js/sha256';
import * as cryptoEncHex from 'crypto-js/enc-hex';
import { v4 as uuidv4 } from 'uuid';
import axios from 'axios';
import { ethers } from 'ethers';
import lock_abi from '../../../../networks/ethereum/abi/HashedTimelock.json';
import { formatEther, parseUnits } from 'ethers/lib/utils';

export default function LockFunds(props) {
  console.log(props.swapDetails);

  let ethereum_lock_contract_address: string = import.meta.env.VITE_ETHEREUM_LOCK_CONTRACT;

  const fromToken = props.swapDetails.fromToken;
  const fromTokenAmount = props.swapDetails.fromTokenAmount;
  const toToken = props.swapDetails.toToken;
  const toTokenAmount = props.swapDetails.bestSwap.expected_balance;
  const provider_address = props.swapDetails.bestSwap.network_address;
  const provider_key = props.swapDetails.bestSwap.public_key;

  function getTokenName(token) {
    switch (token) {
      case "eth.wei":
        return "ether";
      case "tari":
        return "tari";
      default:
        "";
    }
  }

  function formatTokenAmount(token, amount) {
    switch (token) {
      case "eth.wei":
        return formatEther(amount);
      case "tari":
        return amount;
      default:
        0;
    }
  }

  async function getClientAddress() {
    console.log("getClientAddress - toToken: ", toToken);
    switch (toToken) {
      case "eth.wei":
        let provider = new ethers.providers.Web3Provider(window.ethereum);
        provider.send("eth_requestAccounts", [])
        let signer = await provider.getSigner();
        const address = await signer.getAddress();
        console.log("getClientAddress - ethereum address: ", address);
        return address;
      case "tari":
        let res = await window.tari.sendMessage("accounts.get_default", window.tari.token);
        console.log({res});
        let public_key = res.public_key;
        console.log({public_key});
        return public_key;
      default:
        null;
    }
  }

  function getSwapDetails() {
    return (
      <Typography variant="body2">
        {formatTokenAmount(fromToken, fromTokenAmount)} {getTokenName(fromToken)}  <SyncAltIcon sx={{ fontSize: "small", verticalAlign: 'middle' }} /> {toTokenAmount} {getTokenName(toToken)}
      </Typography>
    );
  }

  const lockFunds = async () => {
    console.log("lockFunds");

    let preimage = createPreimage();
    let hashlock = createHashlock(preimage);

    const swap_request_response = await requestSwapFromProvider(provider_address, hashlock);
    console.log({swap_request_response});
    const {swap_id, provider_account_address} = swap_request_response;
    console.log({swap_id, provider_account_address});

    let contract_id_user = await publishLockContract(provider_account_address, fromToken, fromTokenAmount, hashlock);
    console.log({contract_id_user});
    let contract_id_provider = await requestLockFundsFromProvider(provider_address, swap_id, contract_id_user);
    console.log({contract_id_provider});

    // go to the next step in the process
    let payload = {
      swap_id,
      preimage,
      hashlock,
      contract_id_user,
      contract_id_provider,
    };
    console.log({payload});
    props.onCompletion(payload);
  };

  const createPreimage = () => {
    // safe random value
    const random_value = uuidv4();
    // to match the Tari template API, we need the preimage to be a 32 bytes array so we hash it
    const hash = sha256(random_value).toString(cryptoEncHex);
    const hash_as_array = hex_to_int_array(hash);
    return hash_as_array;
  }

  const createHashlock = (preimage) => {
    // depending on the network, we may need to use a different hash algorithm
    const test_preimage_hex = int_array_to_hex(preimage);
    let bytes = CryptoJS.enc.Hex.parse(test_preimage_hex);
    const hash = sha256(bytes).toString(cryptoEncHex);
    const hash_as_array = hex_to_int_array(hash);
    return hash_as_array;
  }

  function int_array_to_hex(int_array) {
    return Array.from(int_array, function(byte) {
      return ('0' + (byte & 0xFF).toString(16)).slice(-2);
    }).join('')
  }

  const hex_to_int_array = (hex_string) => {
    var tokens = hex_string.match(/[0-9a-z]{2}/gi);  // splits the string into segments of two including a remainder => {1,2}
    var int_array = tokens.map(t => parseInt(t, 16));
    return int_array;
  }

  const requestSwapFromProvider = async (provider_address, hashlock) => {
    console.log("requestSwapFromProvider");
    console.log({provider_address, hashlock});

    const client_address = await getClientAddress();
    //const hashlock_array = hex_to_int_array(hashlock);

    const body = {
      jsonrpc: "2.0",
      method: "request_swap",
      id: 1,
      params: {
        client_address,
        hashlock,
        position: {
          provided_token: fromToken,
          provided_token_balance: fromTokenAmount,
          requested_token: toToken,
          requested_token_balance: toTokenAmount.toString(),
        }
      },
    };

    try {
      const response = await axios.post(`${provider_address}/json_rpc`, body);
      console.log("success");
      console.log({response});
      const swap_id = response.data.result.swap_id;
      const provider_account_address = response.data.result.provider_address;
      return {swap_id, provider_account_address};
    } catch (error) {
      console.log("error");
      console.log({error});
    }

    return null;
  }

  const publishLockContract = async (provider_account, fromToken, fromTokenAmount, hashlock) => {
    console.log("publishLockContract");
    console.log({provider_account, fromToken, fromTokenAmount, hashlock});
    switch (fromToken) {
      case "eth.wei":
        console.log("publishLockContract - eth");
        const provider = new ethers.providers.Web3Provider(window.ethereum);
        const signer = await provider.getSigner();
        const contract = new ethers.Contract(ethereum_lock_contract_address, lock_abi, signer);
        //const contractWithSigner = contract.connect(signer);
        console.log({contract});
        const timelock = build_timelock();
        console.log({timelock});
        const provider_address = ethers.utils.getAddress(provider_account);
        //const hashlock_array = hex_to_int_array(hashlock);
        const value = parseUnits(fromTokenAmount, "wei");
        console.log({value});
        const transaction = await contract.newContract(provider_address, hashlock, timelock, {value});
        const transactionReceipt = await transaction.wait();
        console.log({transactionReceipt});
        // In solidity the first topic is the hash of the signature of the event
        // So "contractId" will be in second place on the topics of the "LogHTLCNew" event
        const lock_id = transactionReceipt.logs[0].topics[1];
        console.log({lock_id});
        return lock_id;
      case "tari":
        console.log("publishLockContract - tari");
        let res = await window.tari.sendMessage("keys.list", window.tari.token);
        return res
      default:
        null;
    }
    // the return will be an identifier of the deployed lock contract in the target network
    let contract_id = "contract_id_user";
    return contract_id;
  }

  const build_timelock = () => {
    const secondsSinceEpoch = Math.round(Date.now() / 1000);
    // TODO: make this an environment variable 
    return secondsSinceEpoch + 100;
  }

  const requestLockFundsFromProvider = async (provider_address, swap_id, contract_id_user) => {
    console.log("requestLockFundsFromProvider");
    console.log({provider_address, swap_id, contract_id_user});

    const body = {
      jsonrpc: "2.0",
      method: "request_lock_funds",
      id: 1,
      params: {
        swap_id,
        contract_id: contract_id_user
      },
    };

    try {
      const response = await axios.post(`${provider_address}/json_rpc`, body);
      console.log({response});
      return response.data.result.contract_id;
    } catch (error) {
      console.log({error});
    }

    return null;
  }

  return (
    <div>
      <React.Fragment>
        <List disablePadding>
          <ListItem sx={{ py: 1, px: 0 }}>
            <ListItemText primary="Swap details" />
            {getSwapDetails()}
          </ListItem>
          <ListItem sx={{ py: 1, px: 0 }}>
            <ListItemText primary="Liquidity provider address" />
            <Typography variant="body2">{provider_address}</Typography>
          </ListItem>
          <ListItem sx={{ py: 1, px: 0 }}>
            <ListItemText primary="Liquidity provider public key" />
            <Typography variant="body2">{provider_key}</Typography>
          </ListItem>
        </List>

        <Stack direction="row" alignItems="center" justifyContent="center" sx={{ marginTop: 4 }}>
          <Typography variant="body1">You need to lock <Box component="span" fontWeight='fontWeightMedium'>{formatTokenAmount(fromToken, fromTokenAmount)} {getTokenName(fromToken)}</Box> to begin the atomic swap</Typography>
        </Stack>

        <Box sx={{ display: 'flex', justifyContent: 'center' }}>
          <Button
            variant="contained"
            onClick={lockFunds}
            sx={{ mt: 3, ml: 1, borderRadius: 8, textTransform: 'none' }}
          > Lock funds
          </Button>
        </Box>
      </React.Fragment>
    </div>
  );
}
