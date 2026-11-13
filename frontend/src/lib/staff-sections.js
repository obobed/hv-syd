export const staffSections = [
    { id: 'review', label: 'Review', to: '/dashboard' },
    { id: 'roster', label: 'Roster', to: '/dashboard?view=roster' },
    { id: 'access', label: 'Access', to: '/dashboard?view=access' },
]

export function readStaffSection(search) {
    const id = new URLSearchParams(search).get('view')
    return staffSections.find(section => section.id === id)?.id ?? 'review'
}
