import * as React from 'react';
import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';
import Box from '@mui/material/Box';
import Paper from '@mui/material/Paper';
import { useLocation } from 'react-router-dom';

export default function SwapSteps() {
  const location = useLocation();
  console.log(location.state);
  console.log(window.tari);
  console.log(window.ethereum);  

  return (
    <div>
      <Container maxWidth="sm">
        <Box sx={{ my: 10 }}>
          <Paper variant="outlined" elevation={0} sx={{ my: { xs: 3, md: 6 }, p: { xs: 2, md: 3 }, borderRadius: 8 }}>
            <Box sx={{ my: 1, mx: 1.5 }}>
              <Typography component="h1" variant="h6">
                Swap Steps
              </Typography>
            </Box>
          </Paper>
        </Box>
      </Container>

    </div>
  );
}
