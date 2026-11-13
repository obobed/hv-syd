import { useNavigate } from 'react-router'
import { useEffect, useState } from 'react'
import { Scanner, useDevices } from '@yudiel/react-qr-scanner';
import './misc.css'

export default function Login() {
    const navigate = useNavigate()
    const [attendID, setAttendID] = useState('')
    const devices = useDevices();
    const [selectedDevice, setSelectedDevice] = useState(null);
    const [isPaused, setIsPaused] = useState(false);
    const [username, setUsername] = useState('')
    const [password, setPassword] = useState('')
    const [adminToken, setAdminToken] = useState('')
    const [isAdmin, setIsAdmin] = useState(false)
    const [isLoggedIn, setIsLoggedIn] = useState(false)
    const [error, setError] = useState('')

    useEffect(() => {
      // Check if user is already logged in
      if (isLoggedIn) {
        navigate('/dashboard', {replace: true})
      } 
    }, [isLoggedIn, navigate])

    const handleScan = (detectedCodes) => {
      detectedCodes.forEach(code => {
        const QRID = String(code.rawValue).slice(17)
        setAttendID(QRID)
        console.log({attendID});

        attendeeLogin('/api/v1/login', {
          id: QRID
        })
    });
    };

    async function attendeeLogin(url = '', payload = {}) {
      try {
        const response = await fetch (url, {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          },
          credentials: 'include',
          body: JSON.stringify(payload)
        })

        if (!response.ok) {
            throw new Error(`HTTP error! Status: ${response.status}`)
        }

        setIsAdmin(false)
        setIsLoggedIn(true)

      } catch {
          console.error('Fetch failed:', error.message)
          setIsLoggedIn(false)
      }
    }

    async function adminLogin(url = '', payload = {}) {
      console.log({username, password, adminToken})

      try {
        const response = await fetch(url, {
          method: "POST", 
          headers: {
            "Content-Type": "application/json",
          },
          credentials: 'include',
          body: JSON.stringify(payload)
        })

        if (!response.ok) {
            throw new Error(`HTTP error! Status: ${response.status}`)
        }
        
        //console.log("Login successful! The admin-token cookie is now set.")
        //console.log(response)
        setIsAdmin(true)
        setIsLoggedIn(true)
      } catch (error) {
          console.error('Fetch failed:', error.message)
          setIsAdmin(false)
          setIsLoggedIn(false)
      }
    }

    return(
    <div>

      <button onClick={() => setIsAdmin(!isAdmin)}>
        {isAdmin ? 'Switch to Attendee' : 'Switch to Admin'}
      </button>

      {!isAdmin && (
        <>
        <div className='qrScanner'>
          <Scanner
            onScan={handleScan}
            paused={isPaused}
          />
        </div>

        <div className='qrButtons'>
          <select onChange={(e) => setSelectedDevice(e.target.value)}>
            <option value="">Select a camera</option>
            {devices.map((device) => (
              <option key={device.deviceId} value={device.deviceId}>
                {device.label || `Camera ${device.deviceId}`}
              </option>
            ))}
          </select>

          <button onClick={() => setIsPaused(!isPaused)}>
            {isPaused ? 'Resume' : 'Pause'} Scanning
          </button>
        </div>

        <div className='qrButtons'>
          <input
            type='text'
            placeholder='Enter Attend-ID'
            value={attendID}
            onChange={(e) => setAttendID(e.target.value)}
            size="34"
          />

          <button onClick={() => handleScan}>
            Log In
          </button>
        </div>
      </>
      )}

      {isAdmin && (
        <>
        <div>
          <input
            type='text'
            placeholder='Username'
            value={username}
            onChange={(e) => setUsername(e.target.value)}
          />

          <input
            type='password'
            placeholder='Password'
            value={password}
            onChange={(e) => setPassword(e.target.value)}
          />
        </div>

        <p>OR</p>

        <div>
          <input
            type='text'
            placeholder='Token'
            value={adminToken}
            onChange={(e) => setAdminToken(e.target.value)}
          />
        </div>

        <button
          type='button'
          onClick={() =>
            adminLogin("/api/admin/login", 
              adminToken
                ? { token: adminToken}
                : {
                  username: username,
                  password: password
                }
            )
          }
        >
          Confirm
        </button>
        </>
      )}
    </div>
    )
}