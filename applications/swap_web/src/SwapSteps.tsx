import * as React from 'react';
import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';
import Box from '@mui/material/Box';
import Paper from '@mui/material/Paper';

export default function SwapSteps() {
  

  return (
    <div>
      <Container maxWidth="sm">
        <Box sx={{ my: 10 }}>
          <Paper variant="outlined" elevation={0} sx={{ my: { xs: 3, md: 6 }, p: { xs: 2, md: 3 }, borderRadius: 8 }}>
            <Box sx={{ my: 1, mx: 1.5 }}>
              <Typography component="h1" variant="h6">
                SwapSteps
              </Typography>
            </Box>
          </Paper>
        </Box>
      </Container>

    </div>
  );
}
