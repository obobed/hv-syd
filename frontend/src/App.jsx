import { useState } from 'react'
import { BrowserRouter, Routes, Route } from 'react-router'
import Navbar from './components/navbar'
import Login from './pages/login'
import './App.css'

function App() {

  return (
    <BrowserRouter>
      <Navbar />
      <Routes>
        <Route path='/login' element={<Login />} />
      </Routes>
    </BrowserRouter>
  )
}

export default App
