import { Route, Routes } from "react-router-dom";
import SwapForm from "./SwapForm";
import React from "react";
import SwapSteps from "./steps/SwapSteps";
import ConnectTari from "./ConnectTari";

export default function App() {
  return (
    <div className="App">
      <Routes>
        <Route path="/" element={<ConnectTari />} />
        <Route path="/swap" element={<SwapForm />} />
        <Route path="/steps" element={<SwapSteps />} />
      </Routes>
    </div>
  );
}