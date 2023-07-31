import * as React from 'react';
import { createTheme, ThemeProvider } from '@mui/material/styles';
import AppBar from '@mui/material/AppBar';
import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';
import Box from '@mui/material/Box';
import ProTip from './ProTip';
import Copyright from './Copyright';
import Toolbar from '@mui/material/Toolbar';
import Link from '@mui/material/Link';
import Button from '@mui/material/Button';

const defaultTheme = createTheme();

export default function App() {
  return (
    <ThemeProvider theme={defaultTheme}>
    <AppBar
        position="static"
        color="default"
        elevation={0}
        sx={{ borderBottom: (theme) => `1px solid ${theme.palette.divider}` }}
      >
        <Toolbar sx={{ flexWrap: 'wrap' }}>
          <Typography variant="h6" color="inherit" noWrap sx={{ flexGrow: 1 }}>
            Tari atomic swap
          </Typography>
          <Button href="#" variant="contained" sx={{ my: 1, mx: 1.5 }}>
            Connect wallet
          </Button>
        </Toolbar>
      </AppBar>


    <Container maxWidth="sm">
      <Box sx={{ my: 4 }}>
        <Typography variant="h4" component="h1" gutterBottom>
          Material UI Vite.js example
        </Typography>
        <ProTip />
        <Copyright />
      </Box>
    </Container>
    </ThemeProvider>
  );
}
