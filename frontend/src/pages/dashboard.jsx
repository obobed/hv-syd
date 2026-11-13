import { useNavigate } from 'react-router'
import Cookies from 'js-cookie'
import { useEffect, useState } from 'react'

export default function Dashboard() {
    const [isAdmin, setIsAdmin] = useState(false)
    const [error, setError] = useState('')
    const [tokens, setTokens] = useState({})
    const [ascending, setAscending] = useState(false)
    const [tokenSeconds, setTokenSeconds] = useState(0)
    const [tokenNanoseconds, setTokenNanoseconds] = useState(0)
    const [targetToken, setTargetToken] = useState('')
    const [teamName, setTeamName] = useState('')
    const [hasTeam, setHasTeam] = useState(false)
    const [teamInfo, setTeamInfo] = useState({})
    const [teamOTP, setTeamOTP] = useState('')

    const tokenLevel = {
        ADMIN: 'admin',
        ATTENDEE: 'attendee'
    }

    useEffect(() => {
        teamCheck('/api/v1/team/get')
    }, [hasTeam])

    useEffect(() => {
        getTokens('api/admin/token/list')

        if (isAdmin) {
            // list of all submissions ever
        } else {
            // list of all submissions (for a specific team)
        }
    }, [isAdmin])

    async function joinTeam(url = '', payload = {}) {
        try {
            const response = await fetch(url, {
                method: 'POST',
                headers: {
                    "Content-Type": "application/json"
                },
                credentials: "include",
                body: JSON.stringify(payload)
            })

            if (!response.ok) {
                throw new Error(`HTTP error: ${response.status}`);
            }

            setHasTeam(true)

        } catch {
            console.error(error);
        }
    }

    async function requestOTP(url = '') {
        try {
            const response = await fetch(url, {
                method: 'POST',
                credentials: 'include'
            })

            if (!response.ok) {
                setHasTeam(false)
                throw new Error(`HTTP error: ${response.status}`);
            }

            const otp = response.text()
            setTeamOTP(otp)

        } catch {
            console.error(error);
        }
    }

    async function teamCheck(url = '') {
        try {
            const response = await fetch(url, {
                method: "GET",
                credentials: 'include'
            })

            if (!response.ok) {
                if (response.status == 404) {
                    setHasTeam(false)
                }
                throw new Error(`HTTP error: ${response.status}`);
            }

            setHasTeam(true)
            const rawTeamInfo = await response.JSON()
            setTeamInfo(rawTeamInfo)

        } catch {
            console.error(error);
        }
    }

    async function createTeam(url = '', payload = {}) {
        try {
            const response = await fetch(url, {
                method: "POST",
                headers: {
                    "Content-Type": "application/json"
                },
                credentials: "include",
                body: JSON.stringify(payload)
            })
            if (!response.ok) {
                throw new Error(`HTTP error: ${response.status}`);
            }

            setHasTeam(true)

        } catch {
            console.error(error);
        }
    }

    async function createToken(url = '', payload = {}) {
        console.log(url, payload)

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
                throw new Error(`HTTP error: ${response.status}`);
            }

            const token = await response.text()
            // console.log("token")
            // console.log(token)
            getTokens('/api/admin/token/list')

        } catch {
            console.error(error);
        }
    }

    async function getTokens(url = '') {
        console.log(url)

        try {
            const response = await fetch(url, {
                method: "GET",
                credentials: 'include'
            })

            if (!response.ok) {
                throw new Error(`HTTP error! Status: ${response.status}`)
            }

            const tokenList = await response.json()
            setTokens(tokenList)
            console.log(tokens)

        } catch {
            console.error('Fetch failed', error.message)
        }
    }

    async function tokenUpdate(url = '', payload = {}) {
        try {
            const response = await fetch(url, {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                credentials: "include",
                body: JSON.stringify(payload)
            })

            if (!response.ok) {
                throw new Error (`HTTP error! Status: ${response.status}`)
            }

            getTokens('api/admin/token/list')

        } catch {
            console.log('error')
        }
    }

    return(
        <>
        <button onClick={() => setIsAdmin(!isAdmin)}>
            {isAdmin ? 'Switch to Attendee' : 'Switch to Admin'}
        </button>

        {isAdmin && (
            <>
                <div>
                    <p>unreviewed submissions</p>
                </div>
                <div>
                    <dialog id='tokenCreator'>
                        <button onClick={() => 
                            createToken('api/admin/token/create', 
                                {
                                    level: tokenLevel.ADMIN,
                                    valid_for: [Number(tokenSeconds), Number(tokenNanoseconds)]
                                }
                            )
                        }>
                            Create Token
                        </button>
                        <input
                            type='number'
                            placeholder='Seconds'
                            value={tokenSeconds}
                            onChange={(e) => setTokenSeconds(e.target.value)}
                        />
                        <input
                            type='number'
                            placeholder='Nanoseconds'
                            value={tokenNanoseconds}
                            onChange={(e) => setTokenNanoseconds(e.target.value)}
                        />
                        <button command='close' commandfor='tokenCreator'>
                            X
                        </button>
                    </dialog>

                    <button command='show-modal' commandfor='tokenCreator'>
                        Create New Token
                    </button>

                    <button onClick={() =>
                        getTokens('/api/admin/token/list')
                    }
                    >
                        Refresh Tokens
                    </button>

                    <button onClick={() => setAscending(!ascending)}>
                        {ascending ? 'Expires First' : 'Expires Last'}
                    </button>
                    <table>
                        <thead>
                            <tr>
                                <th>Token ID</th>
                                <th>Expiry</th>
                            </tr>
                        </thead>
                        <tbody>
                            {Object.entries(tokens)
                                .sort(([, a], [, b]) => 
                                    ascending
                                    ? b.exp.localeCompare(a.exp)
                                    : a.exp.localeCompare(b.exp)
                                )
                                .map(([tokenId, metadata]) => (
                                <tr key={tokenId}>
                                    <td><code>{tokenId}</code></td>
                                    <td>{new Date(metadata.exp).toLocaleString()}</td>
                                    <button onClick={() => setTargetToken(tokenId)} command='show-modal' commandfor='tokenUpdator'> Edit </button>
                                </tr>
                            ))}
                        </tbody>
                    </table>

                    <dialog id='tokenUpdator'>
                        <p> Updating token {targetToken}</p>

                        <button onClick={() => tokenUpdate(`/api/admin/token/${targetToken}`, {
                            add: [Number(tokenSeconds), Number(tokenNanoseconds)]
                        })
                        } command='close' commandfor='tokenUpdator'>
                            Extend Token
                        </button>

                        <input
                            type='number'
                            placeholder='Seconds'
                            value={tokenSeconds}
                            onChange={(e) => setTokenSeconds(e.target.value)}
                        />
                        <input
                            type='number'
                            placeholder='Nanoseconds'
                            value={tokenNanoseconds}
                            onChange={(e) => setTokenNanoseconds(e.target.value)}
                        />
                        <button command='close' commandfor='tokenUpdator'>
                            X
                        </button>
                    </dialog>
                </div>
            </>
        )}

        {!isAdmin && (
            <>
                <div>
                    <p>list of submissions made + button to see all milestones</p>
                </div>
                {hasTeam && (
                    <div>
                        <p>{teamOTP}</p>
                        <p>"uhhh convert the teamInfo into an array so it can go here nealy (list names of team mebbers, and team name)"</p>
                        <button onClick={() => {
                            requestOTP('/api/v1/team/otp')
                        }}
                        >
                            Make an OTP Code so your friends can join!
                        </button>
                    </div>
                )}

                {!hasTeam &&(
                    <div>
                        <input
                            type='text'
                            placeholder='Input OTP Code!'
                            value={teamOTP}
                            onChange={(e) => setTeamOTP(e.target.value)}
                        />
                        <button onClick={() => {
                            joinTeam('/api/v1/team/join', {
                                otp: teamOTP
                            })
                        }}>
                            Join!
                        </button>
                        <p> OR </p>
                        <input
                            type='text'
                            placeholder='Team Name'
                            value={teamName}
                            onChange={(e) => setTeamName(e.target.value)}
                        />
                        <button onClick={() => {
                            createTeam('/api/v1/team/create', {
                                teamName: teamName
                            })
                        }}>
                            Create a new team
                        </button>
                    </div>
                )}
            </>
        )}
        </>
    )
}