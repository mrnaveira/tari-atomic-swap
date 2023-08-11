import * as React from 'react';
import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';
import Box from '@mui/material/Box';
import Paper from '@mui/material/Paper';
import { useLocation } from 'react-router-dom';
import AppBar from '@mui/material/AppBar';
import Toolbar from '@mui/material/Toolbar';
import Stack from '@mui/material/Stack';
import Stepper from '@mui/material/Stepper';
import StepLabel from '@mui/material/StepLabel';
import Step from '@mui/material/Step';
import Button from '@mui/material/Button';
import LockFunds from './LockFunds';
import Withdraw from './Withdraw';
import Summary from './Summary';
import { useNavigate } from "react-router-dom";

const steps = ['Lock funds', 'Withdraw', 'Summary'];

export default function SwapSteps() {
  const location = useLocation();
  console.log(location.state);
  console.log(window.tari);
  console.log(window.ethereum);

  const navigate = useNavigate();

  const [activeStep, setActiveStep] = React.useState(0);

  function getStepContent(step: number) {
    switch (step) {
      case 0:
        return <LockFunds />;
      case 1:
        return <Withdraw />;
      case 2:
        return <Summary />;
      default:
        navigate("/");
    }
  }

  const handleNext = () => {
    setActiveStep(activeStep + 1);
  };

  return (
    <div>
      <AppBar
        position="static"
        color="transparent"
        elevation={0}
      >
        <Toolbar sx={{ justifyContent: 'space-between' }}>
          <Stack direction='row'>
            <img width="35px" height="35px" src="/content/tari-logo.svg" />
            <Typography variant="h6" color="inherit" noWrap sx={{ mx: 1 }}>
              Tari Atomic Swap
            </Typography>
          </Stack>
        </Toolbar>
      </AppBar>

      <Container maxWidth="sm">
        <Box sx={{ my: 10 }}>
          <Paper variant="outlined" elevation={0} sx={{ my: { xs: 3, md: 6 }, p: { xs: 2, md: 3 }, borderRadius: 8 }}>
            <Box sx={{ my: 1, mx: 1.5 }}>
              <Stack direction="row" alignItems="center" justifyContent="center">
                <Typography component="h1" variant="h6">
                  Swap
                </Typography>
              </Stack>
              <Stack direction="row" alignItems="center" justifyContent="center">
                <Stepper activeStep={activeStep} sx={{ pt: 3, pb: 5 }}>
                  {steps.map((label) => (
                    <Step key={label}>
                      <StepLabel>{label}</StepLabel>
                    </Step>
                  ))}
                </Stepper>
              </Stack>
              <React.Fragment>
                  {getStepContent(activeStep)}
                  <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}>
                    <Button
                      variant="contained"
                      onClick={handleNext}
                      sx={{ mt: 3, ml: 1 }}
                    >
                      {activeStep === steps.length - 1 ? 'Place order' : 'Next'}
                    </Button>
                  </Box>
                </React.Fragment>
            </Box>
          </Paper>
        </Box>
      </Container>

    </div>
  );
}
