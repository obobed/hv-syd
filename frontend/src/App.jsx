import { useEffect, useState } from 'react'
import { BrowserRouter, Navigate, Outlet, Route, Routes, useLocation } from 'react-router'
import HavenScene from './components/haven-scene'
import Navbar from './components/navbar'
import Login from './pages/login'
import Dashboard from './pages/dashboard'
import { readSession } from './lib/session'
import './App.css'

function WithNavbar() {
  const { pathname } = useLocation()
  const [session, setSession] = useState(undefined)

  useEffect(() => {
    let request
    function refresh() {
      request?.abort()
      const controller = new AbortController()
      request = controller
      void readSession(controller.signal).then(result => {
        if (!controller.signal.aborted) setSession(result)
      })
    }

    refresh()
    window.addEventListener('focus', refresh)
    return () => {
      request?.abort()
      window.removeEventListener('focus', refresh)
    }
  }, [pathname])

  return (
    <div className="haven-app-shell">
      <HavenScene />
      <Navbar session={session} />
      <Outlet context={session} />
    </div>
  )
}

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path='/' element={<Login />} />
        <Route element={<WithNavbar />}>
          <Route path='/dashboard' element={<Dashboard />} />
          <Route path='/rollcall' element={<Navigate to="/dashboard?view=roster" replace />} />
        </Route>
      </Routes>
    </BrowserRouter>
  )
}

export default App
