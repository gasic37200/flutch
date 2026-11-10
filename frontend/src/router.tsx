import { createBrowserRouter } from 'react-router'
import AuthShell from './features/auth/components/AuthShell.tsx'
import BroadcastFormPage from './pages/BroadcastFormPage.tsx'
import AuthPage from './pages/AuthPage.tsx'
import ErrorPage from './pages/ErrorPage.tsx'
import ExplorePage from './pages/ExplorePage.tsx'
import HomePage from './pages/HomePage.tsx'
import LegalPage from './pages/LegalPage.tsx'
import MePage from './pages/MePage.tsx'
import ManagePage from './pages/ManagePage.tsx'
import PulsePage from './pages/PulsePage.tsx'
import ReplayListPage from './pages/ReplayListPage.tsx'
import ReplayPage from './pages/ReplayPage.tsx'
import StreamerPage from './pages/StreamerPage.tsx'
import StudioPage from './pages/StudioPage.tsx'
import WalletPage from './pages/WalletPage.tsx'
import WatchPage from './pages/WatchPage.tsx'

// 모든 페이지는 AppShell 안에서 그려지고, 에러 화면(errorElement)도 같은 셸 안에 뜬다.
export const router = createBrowserRouter([
  {
    element: <AuthShell />,
    children: [
      {
        errorElement: <ErrorPage />,
        children: [
          { index: true, element: <HomePage /> },
          { path: 'explore', element: <ExplorePage /> },
          { path: 'live/:broadcastId', element: <WatchPage /> },
          { path: 'manage', element: <ManagePage /> },
          { path: 'broadcasts/new', element: <BroadcastFormPage /> },
          { path: 'broadcasts/:broadcastId/edit', element: <BroadcastFormPage /> },
          { path: 'studio', element: <StudioPage /> },
          { path: 'pulse', element: <PulsePage /> },
          { path: 'replay', element: <ReplayListPage /> },
          { path: 'replay/:recordingId', element: <ReplayPage /> },
          { path: 'users/:userId', element: <StreamerPage /> },
          { path: 'wallet', element: <WalletPage /> },
          { path: 'login', element: <AuthPage mode="login" /> },
          { path: 'signup', element: <AuthPage mode="signup" /> },
          { path: 'me', element: <MePage /> },
          { path: 'terms', element: <LegalPage doc="terms" /> },
          { path: 'privacy', element: <LegalPage doc="privacy" /> },
          { path: '*', element: <ErrorPage /> },
        ],
      },
    ],
  },
])
