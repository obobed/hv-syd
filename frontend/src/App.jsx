import { BrowserRouter, Outlet, Route, Routes } from 'react-router'
import Navbar from './components/navbar'
import Login from './pages/login'
import Dashboard from './pages/dashboard'
import Rollcall from './pages/rollcall'
import './App.css'

function WithNavbar() {
  return (
    <>
      <Navbar />
      <Outlet />
    </>
  )
}

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path='/' element={<Login />} />
        <Route element={<WithNavbar />}>
          <Route path='/dashboard' element={<Dashboard />} />
          <Route path='/rollcall' element={<Rollcall />} />
        </Route>
      </Routes>
    </BrowserRouter>
  )
}

export default App
