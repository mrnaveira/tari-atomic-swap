import { Route, Routes } from "react-router-dom";
import SwapForm from "./SwapForm";
import React from "react";
import SwapSteps from "./steps/SwapSteps";

export default function App() {
  return (
    <div className="App">
      <Routes>
        <Route path="/" element={<SwapForm />} />
        <Route path="/steps" element={<SwapSteps />} />
      </Routes>
    </div>
  );
}