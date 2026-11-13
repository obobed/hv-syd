import { useNavigate } from 'react-router'
import { useEffect, useState } from 'react'
import { Scanner, useDevices } from '@yudiel/react-qr-scanner';
import Cookies from 'js-cookie';

export default function Rollcall() {
    const devices = useDevices();
    const [isPaused, setIsPaused] = useState(false);
    const [attendID, setAttendID] = useState('')
    const [scanContext, setScanContext] = useState(0)
    const [attendeeData, setAttendeeData] = useState([])
    const [teamData, setTeamData] = useState([])
    const [viewAttendees, setViewAttendees] = useState(true)


    useEffect(() => {
        const loginToken = Cookies.get('admin-token')
        console.log(loginToken)
    }, [])

    const handleScan = (detectedCodes) => {
      detectedCodes.forEach(code => {
        const QRID = String(code.rawValue).slice(17)
        setAttendID(QRID)
        console.log({attendID});

        checkIn({
            participant_id: QRID,
            scan_context_id: scanContext,
            badge_token: "",
            client_scan_id: "",
            source: "manual",
            scanned_at: new Date().toISOString()

        })
        // Send ID to API and return login result. Set isLoggedIn to true if successful
    })
    }

    async function checkIn(payload = {}) {
        try {
            const response = await fetch('/attend/api/v1/events/haven-sydney/scans', {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                }
            })
        } catch {

        }
    }

    return(
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

                <button onClick={() =>  checkIn({
                    participant_id: attendID,
                    scan_context_id: scanContext,
                    badge_token: "",
                    client_scan_id: "",
                    source: "manual",
                    scanned_at: new Date().toISOString()

                })}>
                    Register
                </button>
            </div>
            <div>

                <button onClick={() => setViewAttendees(!viewAttendees)}>
                    {viewAttendees ? "View teams" : "View Attendees"}
                </button>

                {viewAttendees && (
                    <div>
                        <table>
                            <thead>
                                <tr>
                                    <th>ID</th>
                                    <th>FirstN</th>
                                    <th>LastN</th>
                                </tr>
                            </thead>
                            <tbody>
                                {attendeeData.map(person => (
                                    <tr key={person.id}>
                                        <td>{person.id}</td>
                                        <td>{person.first_name}</td>
                                        <td>{person.last_name}</td>
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    </div>
                )}

                {!viewAttendees && (
                    <div>
                        <table>
                            <thead>
                                <tr>
                                    <th>Team Name</th>
                                </tr>
                            </thead>
                            <tbody>
                                {teamData.map((team, index) => (
                                    <tr key={index}>
                                        <td>{team.team_name}</td>
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    </div>
                )}
            </div>
        </>
    )
}