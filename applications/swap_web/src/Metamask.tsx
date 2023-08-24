import React, { useState } from 'react';
import Button from '@mui/material/Button';
import { ethers } from 'ethers';

let signer = null;
let provider;

const Metamask = (props) => {
    const [account, setAccount] = useState(null);
    const [balance, setBalance] = useState(null);

    const connectwalletHandler = async() => {
        if (window.ethereum) {
            provider = new ethers.providers.Web3Provider(window.ethereum);
            provider.send("eth_requestAccounts", [])
                .then(async () => {
                    signer = await provider.getSigner();
                    await accountChangedHandler(signer);
            });
        } else {
            console.log("MetaMask not installed; using read-only defaults");
            provider = ethers.getDefaultProvider(import.meta.env.VITE_ETHEREUM_RPC_URL);
        }
    }

    const accountChangedHandler = async (signer) => {
        console.log({signer});
        setAccount(signer);

        const address = await signer.getAddress();
        const balance = await provider.getBalance(address);
        setBalance(balance);
        console.log(ethers.utils.formatEther(balance));
        props.onConnection({balance});
    }

    return (
        <Button variant="contained" sx={{ my: 1, mx: 1.5, borderRadius: 8, textTransform: 'none' }}  
            onClick={connectwalletHandler}>
            {account ? "Connected" : "Connect Metamask"}
          </Button>
    )
}
export default Metamask;


