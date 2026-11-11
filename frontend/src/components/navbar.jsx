import { Link, useNavigate } from 'react-router'
import './navbar.css'
import logo from '../assets/hvsydLogoTrans.svg'

export default function Navbar() {
    const navigate = useNavigate()

    return (
        <nav className='navbg'>
            <img src= {logo} alt='hvsyd' className='logo' /> 
            <p1 className='navlink'>hello!</p1>
        </nav>
    )
}