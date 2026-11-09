import { createBrowserRouter } from 'react-router'
import AuthPage from './pages/AuthPage.tsx'
import ErrorPage from './pages/ErrorPage.tsx'
import ExplorePage from './pages/ExplorePage.tsx'
import HomePage from './pages/HomePage.tsx'
import ReplayPage from './pages/ReplayPage.tsx'
import StreamerPage from './pages/StreamerPage.tsx'
import StudioPage from './pages/StudioPage.tsx'
import WalletPage from './pages/WalletPage.tsx'
import WatchPage from './pages/WatchPage.tsx'

// 페이지 라우트 자리. 각 화면 task는 자기 페이지 컴포넌트만 채우고, 레이아웃(AppShell)은 T05가 감싼다.
export const router = createBrowserRouter([
  {
    path: '/',
    errorElement: <ErrorPage />,
    children: [
      { index: true, element: <HomePage /> },
      { path: 'explore', element: <ExplorePage /> },
      { path: 'live/:broadcastId', element: <WatchPage /> },
      { path: 'studio', element: <StudioPage /> },
      { path: 'replay/:recordingId', element: <ReplayPage /> },
      { path: 'users/:userId', element: <StreamerPage /> },
      { path: 'wallet', element: <WalletPage /> },
      { path: 'login', element: <AuthPage mode="login" /> },
      { path: 'signup', element: <AuthPage mode="signup" /> },
      { path: '*', element: <ErrorPage /> },
    ],
  },
])
