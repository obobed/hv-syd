import { useNavigate } from 'react-router'
import Cookies from 'js-cookie'
import { useEffect, useState } from 'react'

export default function Dashboard() {
    const [isAdmin, setIsAdmin] = useState(false)
    const [hasTeam, setHasTeam] = useState(false) 
    const [submissions, setSubmissions] = useState([])
    const [tokenValid, setTokenValid] = useState([])
    const [error, setError] = useState('')

    const tokenLevel = {
        ADMIN: 'admin'
    }

    useEffect(() => {
        if (isAdmin) {
            // list of all submissions ever
        } else {
            // list of all submissions (for a specific team)
        }
    })

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
            console.log(token)

        } catch {
            console.error(error);
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
                    <button onClick={() => 
                        createToken('api/admin/token/create', 
                            {
                                level: tokenLevel.ADMIN,
                                valid_for: tokenValid
                            }
                        )
                    }>
                        Create Token
                    </button>
                </div>
            </>
        )}

        {!isAdmin && (
            <>
                <div>
                    <p>list of submissions made + button to see all milestones</p>
                </div>
                <div>
                    <p>team name + member names</p>
                </div>
            </>
        )}
        </>
    )
}