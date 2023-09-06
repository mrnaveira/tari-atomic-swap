import * as React from 'react';
import Typography from '@mui/material/Typography';
import Button from '@mui/material/Button';
import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import axios from 'axios';
import { ethers } from 'ethers';
import lock_abi from '../../../../networks/ethereum/abi/HashedTimelock.json';

import * as tari_lib from '../tari-lib';

export default function Withdraw(props) {
  let ethereum_lock_contract_address: string = import.meta.env.VITE_ETHEREUM_LOCK_CONTRACT;

  const provider_address = props.swapDetails.bestSwap.network_address;
  const expected_balance = props.swapDetails.bestSwap.expected_balance;
  const toToken = props.swapDetails.toToken;
  const contract_id_provider = props.ongoingSwap.contract_id_provider;
  const swap_id = props.ongoingSwap.swap_id;
  const preimage = props.ongoingSwap.preimage;

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

  const withdraw = async () => {
    await withdrawProviderFunds(contract_id_provider, toToken, preimage);
    await pushPreimageToProvider(provider_address, swap_id, preimage);

    props.onCompletion(props.ongoingSwap);
  }

  const withdrawProviderFunds = async (contract_id_provider, toToken, preimage) => {
    // TODO: should return and error if the contract does not exist or does not have the requested amount
    switch (toToken) {
      case "eth.wei":
        const provider = new ethers.providers.Web3Provider(window.ethereum);
        const signer = await provider.getSigner();
        const contract = new ethers.Contract(ethereum_lock_contract_address, lock_abi, signer);
        const transaction = await contract.withdraw(contract_id_provider, preimage);
        const transactionReceipt = await transaction.wait();
        return true;
      case "tari":
        await tari_lib.withdraw(window.tari, contract_id_provider, preimage);
        return true;
      default:
        false;
    }    
  }

  const pushPreimageToProvider = async (provider_address, swap_id, preimage) => {
    const body = {
      jsonrpc: "2.0",
      method: "push_preimage",
      id: 1,
      params: {
        swap_id,
        preimage,
      },
    };

    try {
      await axios.post(`${provider_address}/json_rpc`, body);
    } catch (error) {
      console.log({error});
    }

    return null;
  }
  
  return (
    <div>
      <React.Fragment>
        <Stack direction="row" alignItems="center" justifyContent="center" sx={{ marginTop: 2 }}>
          <Typography variant="body1">The provider has locked <Box component="span" fontWeight='fontWeightMedium'>{expected_balance} {getTokenName(toToken)}</Box> for you. Please proceed to withdraw the funds</Typography>
        </Stack>
        <Box sx={{ display: 'flex', justifyContent: 'center' }}>
          <Button
            variant="contained"
            onClick={withdraw}
            sx={{  mt: 3, ml: 1, borderRadius: 8, textTransform: 'none' }} 
          > Withdraw
          </Button>
        </Box>
      </React.Fragment>
    </div>
  );
}
