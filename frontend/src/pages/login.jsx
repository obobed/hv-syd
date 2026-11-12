import { Link, useNavigate } from 'react-router'
import { useEffect, useState } from 'react'
import { Scanner, useDevices } from '@yudiel/react-qr-scanner';
import './misc.css'

export default function Login() {
    const [attendID, setAttendID] = useState('')
    const devices = useDevices();
    const [selectedDevice, setSelectedDevice] = useState(null);
    const [isPaused, setIsPaused] = useState(false);
    const [username, setUsername] = useState('')
    const [password, setPassword] = useState('')
    const [adminToken, setAdminToken] = useState('')


    const handleScan = (detectedCodes) => {
      console.log('Detected codes:', detectedCodes);
      detectedCodes.forEach(code => {
        const QRID = String(code.rawValue).slice(17)
        setAttendID(QRID)
        console.log({attendID});
    });
    };

    async function adminLogin() {
      console.log({username, password, adminToken})
    }

    return(
    <div>
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

        <button onClick={() => setAttendID(attendID)}>
          Log In
        </button>
      </div>

      <div>
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
          onClick={adminLogin}
        >
          Confirm
        </button>

      </div>
    </div>
    )
}