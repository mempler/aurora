import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';

import { ThemeProvider } from '@/components/theme-provider';

import '../globals.css';

const root = ReactDOM.createRoot(document.getElementById('root')!);
root.render(
  <React.StrictMode>
    <ThemeProvider defaultTheme="dark" storageKey="aurora-ui-theme">
      <App />
    </ThemeProvider>
  </React.StrictMode>,
);
