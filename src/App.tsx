import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { Toaster } from 'react-hot-toast';
import { AnimatePresence } from 'framer-motion';
import Layout from './components/layout/Layout';
import HubPage from './pages/HubPage';
import CollectionPage from './pages/CollectionPage';
import DeckbuilderPage from './pages/DeckbuilderPage';
import DeckDetailPage from './pages/DeckDetailPage';
import LorePage from './pages/LorePage';
import LoreDetailPage from './pages/LoreDetailPage';
import SettingsPage from './pages/SettingsPage';
import DesignShowcase from './pages/DesignShowcase';
import './styles/globals.css';

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 1000 * 60 * 5, // 5 minutes
      retry: 1,
      refetchOnWindowFocus: false,
    },
  },
});

function AppRoutes() {
  return (
    <AnimatePresence mode="wait">
      <Routes>
        <Route path="/" element={<Layout />}>
          <Route index element={<HubPage />} />
          <Route path="collection" element={<CollectionPage />} />
          <Route path="deckbuilder" element={<DeckbuilderPage />} />
          <Route path="deck/:id" element={<DeckDetailPage />} />
          <Route path="lore" element={<LorePage />} />
          <Route path="lore/:id" element={<LoreDetailPage />} />
          <Route path="settings" element={<SettingsPage />} />
        </Route>
        <Route path="/design" element={<DesignShowcase />} />
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </AnimatePresence>
  );
}

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        <AppRoutes />
        <Toaster
          position="bottom-right"
          toastOptions={{
            duration: 4000,
            style: {
              background: 'var(--glass-bg)',
              backdropFilter: 'var(--glass-blur)',
              border: '1px solid var(--glass-border)',
              borderRadius: 'var(--radius-subtle)',
              color: 'var(--color-text-main)',
            },
            success: {
              iconTheme: {
                primary: 'var(--color-parchment)',
                secondary: 'var(--color-bg-base)',
              },
            },
            error: {
              iconTheme: {
                primary: 'var(--color-crimson)',
                secondary: 'var(--color-bg-base)',
              },
            },
          }}
        />
      </BrowserRouter>
    </QueryClientProvider>
  );
}

export default App;