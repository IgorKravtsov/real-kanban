import { BrowserRouter, Routes, Route } from 'react-router-dom'
import ProjectListPage from './pages/ProjectListPage'
import ProjectBoardPage from './pages/ProjectBoardPage'
import { ThemeProvider } from './components/theme-provider'
import { Layout } from './components/Layout'

export default function App() {
  return (
    <ThemeProvider defaultTheme="dark">
      <BrowserRouter>
        <Routes>
          <Route element={<Layout />}>
            <Route path="/" element={<ProjectListPage />} />
            <Route path="/projects/:id" element={<ProjectBoardPage />} />
          </Route>
          <Route path="*" element={<NotFoundPage />} />
        </Routes>
      </BrowserRouter>
    </ThemeProvider>
  )
}

function NotFoundPage() {
  return (
    <div style={{ padding: '2rem' }}>
      <h1>404 - Page Not Found</h1>
      <p>The page you are looking for does not exist.</p>
      <a href="/">Go to Home</a>
    </div>
  )
}
