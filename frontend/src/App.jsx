import { useEffect, useState } from 'react'
import { BrowserRouter, Navigate, Outlet, Route, Routes, useLocation, useOutletContext } from 'react-router'
import Navbar from './components/navbar'
import Login from './pages/login'
import Dashboard from './pages/dashboard'
import Rollcall from './pages/rollcall'
import { readSession } from './lib/session'
import './App.css'

function WithNavbar() {
  const { key } = useLocation()
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
  }, [key])

  return (
    <>
      <Navbar session={session} />
      <Outlet context={session} />
    </>
  )
}

function StaffRollcall() {
  const session = useOutletContext()
  if (session === undefined) return <p role="status">Checking your session…</p>
  if (session?.role !== 'staff') return <Navigate to="/dashboard" replace />
  return <Rollcall />
}

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path='/' element={<Login />} />
        <Route element={<WithNavbar />}>
          <Route path='/dashboard' element={<Dashboard />} />
          <Route path='/rollcall' element={<StaffRollcall />} />
        </Route>
      </Routes>
    </BrowserRouter>
  )
}

export default App
