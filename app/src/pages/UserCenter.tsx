import { useState } from 'react'

interface UserProfile {
  id: string
  username: string
  email: string
  balance: number
  frozenBalance: number
  memberSince: string
  totalSpent: number
  totalTransactions: number
}

interface Transaction {
  id: string
  type: 'deposit' | 'withdrawal' | 'purchase' | 'refund'
  amount: number
  description: string
  status: 'completed' | 'pending' | 'failed'
  createdAt: string
}

function UserCenter() {
  const [activeView, setActiveView] = useState<'profile' | 'transactions' | 'wallet'>('profile')
  const [showDeposit, setShowDeposit] = useState(false)
  const [showWithdraw, setShowWithdraw] = useState(false)
  const [depositAmount, setDepositAmount] = useState('')
  const [withdrawAmount, setWithdrawAmount] = useState('')

  // Mock user data
  const [user] = useState<UserProfile>({
    id: 'u123',
    username: 'john_doe',
    email: 'john@example.com',
    balance: 5420,
    frozenBalance: 150,
    memberSince: '2025-06-15',
    totalSpent: 12850,
    totalTransactions: 156
  })

  const [transactions] = useState<Transaction[]>([
    { id: 't1', type: 'purchase', amount: -50, description: 'Data Analyzer Pro', status: 'completed', createdAt: '2026-03-03T10:30:00Z' },
    { id: 't2', type: 'deposit', amount: 1000, description: 'Wallet deposit', status: 'completed', createdAt: '2026-03-02T14:20:00Z' },
    { id: 't3', type: 'purchase', amount: -25, description: 'Code Review Assistant', status: 'completed', createdAt: '2026-03-02T09:15:00Z' },
    { id: 't4', type: 'purchase', amount: -99, description: 'Content Writer AI', status: 'pending', createdAt: '2026-03-01T16:45:00Z' },
    { id: 't5', type: 'withdrawal', amount: -500, description: 'Bank transfer', status: 'completed', createdAt: '2026-02-28T11:30:00Z' },
    { id: 't6', type: 'refund', amount: 30, description: 'Refund for cancelled order', status: 'completed', createdAt: '2026-02-27T08:00:00Z' },
    { id: 't7', type: 'purchase', amount: -199, description: 'Image Generator Pro', status: 'completed', createdAt: '2026-02-26T13:45:00Z' },
    { id: 't8', type: 'deposit', amount: 2000, description: 'Wallet deposit', status: 'completed', createdAt: '2026-02-25T10:20:00Z' }
  ])

  const handleDeposit = () => {
    const amount = parseFloat(depositAmount)
    if (isNaN(amount) || amount <= 0) {
      alert('Please enter a valid amount')
      return
    }
    alert(`Depositing ${amount} PC to your wallet...`)
    setShowDeposit(false)
    setDepositAmount('')
  }

  const handleWithdraw = () => {
    const amount = parseFloat(withdrawAmount)
    if (isNaN(amount) || amount <= 0) {
      alert('Please enter a valid amount')
      return
    }
    if (amount > user.balance) {
      alert('Insufficient balance')
      return
    }
    alert(`Withdrawing ${amount} PC from your wallet...`)
    setShowWithdraw(false)
    setWithdrawAmount('')
  }

  const getTransactionIcon = (type: string) => {
    switch (type) {
      case 'deposit': return '💰'
      case 'withdrawal': return '🏦'
      case 'purchase': return '🛒'
      case 'refund': return '↩️'
      default: return '💳'
    }
  }

  return (
    <div style={{ padding: '20px', maxWidth: '1400px', margin: '0 auto' }}>
      {/* Header */}
      <div style={{ marginBottom: '30px' }}>
        <h1 style={{ fontSize: '32px', marginBottom: '10px' }}>👤 User Center</h1>
        <p style={{ color: '#666', fontSize: '16px' }}>
          Manage your account, wallet, and transaction history
        </p>
      </div>

      {/* View Toggle */}
      <div style={{ display: 'flex', gap: '10px', marginBottom: '20px' }}>
        <button
          onClick={() => setActiveView('profile')}
          style={{
            padding: '10px 20px',
            backgroundColor: activeView === 'profile' ? '#007bff' : '#f0f0f0',
            color: activeView === 'profile' ? 'white' : '#333',
            border: 'none',
            borderRadius: '5px',
            cursor: 'pointer',
            fontSize: '14px',
            fontWeight: activeView === 'profile' ? 'bold' : 'normal'
          }}
        >
          Profile
        </button>
        <button
          onClick={() => setActiveView('wallet')}
          style={{
            padding: '10px 20px',
            backgroundColor: activeView === 'wallet' ? '#007bff' : '#f0f0f0',
            color: activeView === 'wallet' ? 'white' : '#333',
            border: 'none',
            borderRadius: '5px',
            cursor: 'pointer',
            fontSize: '14px',
            fontWeight: activeView === 'wallet' ? 'bold' : 'normal'
          }}
        >
          Wallet
        </button>
        <button
          onClick={() => setActiveView('transactions')}
          style={{
            padding: '10px 20px',
            backgroundColor: activeView === 'transactions' ? '#007bff' : '#f0f0f0',
            color: activeView === 'transactions' ? 'white' : '#333',
            border: 'none',
            borderRadius: '5px',
            cursor: 'pointer',
            fontSize: '14px',
            fontWeight: activeView === 'transactions' ? 'bold' : 'normal'
          }}
        >
          Transactions ({transactions.length})
        </button>
      </div>

      {/* Profile View */}
      {activeView === 'profile' && (
        <div>
          <div style={{
            backgroundColor: 'white',
            padding: '30px',
            borderRadius: '8px',
            border: '1px solid #e0e0e0',
            marginBottom: '20px'
          }}>
            <h2 style={{ marginBottom: '20px' }}>Account Information</h2>
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '20px' }}>
              <div>
                <label style={{ display: 'block', fontSize: '14px', color: '#666', marginBottom: '5px' }}>
                  Username
                </label>
                <input
                  type="text"
                  value={user.username}
                  readOnly
                  style={{
                    width: '100%',
                    padding: '10px',
                    border: '1px solid #ddd',
                    borderRadius: '5px',
                    backgroundColor: '#f8f9fa'
                  }}
                />
              </div>
              <div>
                <label style={{ display: 'block', fontSize: '14px', color: '#666', marginBottom: '5px' }}>
                  Email
                </label>
                <input
                  type="email"
                  value={user.email}
                  readOnly
                  style={{
                    width: '100%',
                    padding: '10px',
                    border: '1px solid #ddd',
                    borderRadius: '5px',
                    backgroundColor: '#f8f9fa'
                  }}
                />
              </div>
              <div>
                <label style={{ display: 'block', fontSize: '14px', color: '#666', marginBottom: '5px' }}>
                  User ID
                </label>
                <input
                  type="text"
                  value={user.id}
                  readOnly
                  style={{
                    width: '100%',
                    padding: '10px',
                    border: '1px solid #ddd',
                    borderRadius: '5px',
                    backgroundColor: '#f8f9fa'
                  }}
                />
              </div>
              <div>
                <label style={{ display: 'block', fontSize: '14px', color: '#666', marginBottom: '5px' }}>
                  Member Since
                </label>
                <input
                  type="text"
                  value={new Date(user.memberSince).toLocaleDateString()}
                  readOnly
                  style={{
                    width: '100%',
                    padding: '10px',
                    border: '1px solid #ddd',
                    borderRadius: '5px',
                    backgroundColor: '#f8f9fa'
                  }}
                />
              </div>
            </div>
            <button
              style={{
                marginTop: '20px',
                padding: '10px 20px',
                backgroundColor: '#007bff',
                color: 'white',
                border: 'none',
                borderRadius: '5px',
                cursor: 'pointer',
                fontSize: '14px'
              }}
              onClick={() => alert('Edit profile functionality coming soon...')}
            >
              Edit Profile
            </button>
          </div>

          {/* Statistics */}
          <div style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fit, minmax(250px, 1fr))',
            gap: '20px'
          }}>
            <div style={{
              backgroundColor: 'white',
              padding: '20px',
              borderRadius: '8px',
              border: '1px solid #e0e0e0'
            }}>
              <div style={{ fontSize: '14px', color: '#666', marginBottom: '8px' }}>Total Spent</div>
              <div style={{ fontSize: '28px', fontWeight: 'bold', color: '#dc3545' }}>
                {user.totalSpent.toLocaleString()} PC
              </div>
            </div>
            <div style={{
              backgroundColor: 'white',
              padding: '20px',
              borderRadius: '8px',
              border: '1px solid #e0e0e0'
            }}>
              <div style={{ fontSize: '14px', color: '#666', marginBottom: '8px' }}>Total Transactions</div>
              <div style={{ fontSize: '28px', fontWeight: 'bold', color: '#007bff' }}>
                {user.totalTransactions}
              </div>
            </div>
            <div style={{
              backgroundColor: 'white',
              padding: '20px',
              borderRadius: '8px',
              border: '1px solid #e0e0e0'
            }}>
              <div style={{ fontSize: '14px', color: '#666', marginBottom: '8px' }}>Account Age</div>
              <div style={{ fontSize: '28px', fontWeight: 'bold', color: '#6f42c1' }}>
                {Math.floor((Date.now() - new Date(user.memberSince).getTime()) / (1000 * 60 * 60 * 24))} days
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Wallet View */}
      {activeView === 'wallet' && (
        <div>
          {/* Balance Cards */}
          <div style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fit, minmax(300px, 1fr))',
            gap: '20px',
            marginBottom: '30px'
          }}>
            <div style={{
              backgroundColor: 'white',
              padding: '30px',
              borderRadius: '8px',
              border: '1px solid #e0e0e0',
              boxShadow: '0 2px 8px rgba(0,0,0,0.1)'
            }}>
              <div style={{ fontSize: '16px', color: '#666', marginBottom: '10px' }}>Available Balance</div>
              <div style={{ fontSize: '36px', fontWeight: 'bold', color: '#28a745', marginBottom: '15px' }}>
                {user.balance.toLocaleString()} PC
              </div>
              <div style={{ fontSize: '14px', color: '#666' }}>
                Frozen: {user.frozenBalance} PC
              </div>
            </div>

            <div style={{
              backgroundColor: 'white',
              padding: '30px',
              borderRadius: '8px',
              border: '1px solid #e0e0e0',
              boxShadow: '0 2px 8px rgba(0,0,0,0.1)'
            }}>
              <div style={{ fontSize: '16px', color: '#666', marginBottom: '10px' }}>Total Balance</div>
              <div style={{ fontSize: '36px', fontWeight: 'bold', color: '#007bff', marginBottom: '15px' }}>
                {(user.balance + user.frozenBalance).toLocaleString()} PC
              </div>
              <div style={{ fontSize: '14px', color: '#666' }}>
                Available + Frozen
              </div>
            </div>
          </div>

          {/* Actions */}
          <div style={{
            backgroundColor: 'white',
            padding: '30px',
            borderRadius: '8px',
            border: '1px solid #e0e0e0',
            marginBottom: '20px'
          }}>
            <h3 style={{ marginBottom: '20px' }}>Wallet Actions</h3>
            <div style={{ display: 'flex', gap: '15px' }}>
              <button
                onClick={() => setShowDeposit(true)}
                style={{
                  flex: 1,
                  padding: '15px',
                  backgroundColor: '#28a745',
                  color: 'white',
                  border: 'none',
                  borderRadius: '5px',
                  cursor: 'pointer',
                  fontSize: '16px',
                  fontWeight: 'bold'
                }}
              >
                💰 Deposit
              </button>
              <button
                onClick={() => setShowWithdraw(true)}
                style={{
                  flex: 1,
                  padding: '15px',
                  backgroundColor: '#007bff',
                  color: 'white',
                  border: 'none',
                  borderRadius: '5px',
                  cursor: 'pointer',
                  fontSize: '16px',
                  fontWeight: 'bold'
                }}
              >
                🏦 Withdraw
              </button>
            </div>
          </div>

          {/* Recent Wallet Transactions */}
          <div style={{
            backgroundColor: 'white',
            padding: '20px',
            borderRadius: '8px',
            border: '1px solid #e0e0e0'
          }}>
            <h3 style={{ marginBottom: '15px' }}>Recent Wallet Activity</h3>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
              {transactions
                .filter(t => t.type === 'deposit' || t.type === 'withdrawal')
                .slice(0, 5)
                .map(tx => (
                  <div
                    key={tx.id}
                    style={{
                      display: 'flex',
                      justifyContent: 'space-between',
                      alignItems: 'center',
                      padding: '15px',
                      backgroundColor: '#f8f9fa',
                      borderRadius: '5px'
                    }}
                  >
                    <div style={{ display: 'flex', alignItems: 'center', gap: '15px' }}>
                      <div style={{ fontSize: '24px' }}>{getTransactionIcon(tx.type)}</div>
                      <div>
                        <div style={{ fontWeight: 'bold' }}>{tx.description}</div>
                        <div style={{ fontSize: '12px', color: '#666' }}>
                          {new Date(tx.createdAt).toLocaleString()}
                        </div>
                      </div>
                    </div>
                    <div style={{
                      fontSize: '18px',
                      fontWeight: 'bold',
                      color: tx.amount > 0 ? '#28a745' : '#dc3545'
                    }}>
                      {tx.amount > 0 ? '+' : ''}{tx.amount} PC
                    </div>
                  </div>
                ))}
            </div>
          </div>
        </div>
      )}

      {/* Transactions View */}
      {activeView === 'transactions' && (
        <div>
          <h2 style={{ marginBottom: '20px' }}>Transaction History</h2>
          <div style={{ backgroundColor: 'white', borderRadius: '8px', overflow: 'hidden', border: '1px solid #e0e0e0' }}>
            <table style={{ width: '100%', borderCollapse: 'collapse' }}>
              <thead>
                <tr style={{ backgroundColor: '#f8f9fa' }}>
                  <th style={{ padding: '15px', textAlign: 'left', borderBottom: '2px solid #dee2e6' }}>Type</th>
                  <th style={{ padding: '15px', textAlign: 'left', borderBottom: '2px solid #dee2e6' }}>Description</th>
                  <th style={{ padding: '15px', textAlign: 'left', borderBottom: '2px solid #dee2e6' }}>Amount</th>
                  <th style={{ padding: '15px', textAlign: 'left', borderBottom: '2px solid #dee2e6' }}>Status</th>
                  <th style={{ padding: '15px', textAlign: 'left', borderBottom: '2px solid #dee2e6' }}>Date</th>
                </tr>
              </thead>
              <tbody>
                {transactions.map(tx => (
                  <tr key={tx.id} style={{ borderBottom: '1px solid #f0f0f0' }}>
                    <td style={{ padding: '15px' }}>
                      <span style={{ fontSize: '20px', marginRight: '8px' }}>{getTransactionIcon(tx.type)}</span>
                      {tx.type.charAt(0).toUpperCase() + tx.type.slice(1)}
                    </td>
                    <td style={{ padding: '15px' }}>{tx.description}</td>
                    <td style={{
                      padding: '15px',
                      fontWeight: 'bold',
                      color: tx.amount > 0 ? '#28a745' : '#dc3545'
                    }}>
                      {tx.amount > 0 ? '+' : ''}{tx.amount} PC
                    </td>
                    <td style={{ padding: '15px' }}>
                      <span style={{
                        padding: '4px 12px',
                        borderRadius: '12px',
                        fontSize: '12px',
                        fontWeight: 'bold',
                        backgroundColor: tx.status === 'completed' ? '#d4edda' : tx.status === 'pending' ? '#fff3cd' : '#f8d7da',
                        color: tx.status === 'completed' ? '#155724' : tx.status === 'pending' ? '#856404' : '#721c24'
                      }}>
                        {tx.status.toUpperCase()}
                      </span>
                    </td>
                    <td style={{ padding: '15px' }}>{new Date(tx.createdAt).toLocaleString()}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Deposit Modal */}
      {showDeposit && (
        <div
          style={{
            position: 'fixed',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            backgroundColor: 'rgba(0,0,0,0.5)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            zIndex: 1000
          }}
          onClick={() => setShowDeposit(false)}
        >
          <div
            style={{
              backgroundColor: 'white',
              padding: '30px',
              borderRadius: '8px',
              maxWidth: '400px',
              width: '90%'
            }}
            onClick={(e) => e.stopPropagation()}
          >
            <h2 style={{ marginBottom: '20px' }}>💰 Deposit Funds</h2>
            <div style={{ marginBottom: '20px' }}>
              <label style={{ display: 'block', fontSize: '14px', color: '#666', marginBottom: '8px' }}>
                Amount (PC)
              </label>
              <input
                type="number"
                value={depositAmount}
                onChange={(e) => setDepositAmount(e.target.value)}
                placeholder="Enter amount"
                style={{
                  width: '100%',
                  padding: '12px',
                  border: '1px solid #ddd',
                  borderRadius: '5px',
                  fontSize: '16px'
                }}
              />
            </div>
            <div style={{ fontSize: '14px', color: '#666', marginBottom: '20px' }}>
              Current balance: {user.balance.toLocaleString()} PC
            </div>
            <div style={{ display: 'flex', gap: '10px' }}>
              <button
                style={{
                  flex: 1,
                  padding: '12px',
                  backgroundColor: '#6c757d',
                  color: 'white',
                  border: 'none',
                  borderRadius: '5px',
                  cursor: 'pointer',
                  fontSize: '14px'
                }}
                onClick={() => setShowDeposit(false)}
              >
                Cancel
              </button>
              <button
                style={{
                  flex: 1,
                  padding: '12px',
                  backgroundColor: '#28a745',
                  color: 'white',
                  border: 'none',
                  borderRadius: '5px',
                  cursor: 'pointer',
                  fontSize: '14px',
                  fontWeight: 'bold'
                }}
                onClick={handleDeposit}
              >
                Deposit
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Withdraw Modal */}
      {showWithdraw && (
        <div
          style={{
            position: 'fixed',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            backgroundColor: 'rgba(0,0,0,0.5)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            zIndex: 1000
          }}
          onClick={() => setShowWithdraw(false)}
        >
          <div
            style={{
              backgroundColor: 'white',
              padding: '30px',
              borderRadius: '8px',
              maxWidth: '400px',
              width: '90%'
            }}
            onClick={(e) => e.stopPropagation()}
          >
            <h2 style={{ marginBottom: '20px' }}>🏦 Withdraw Funds</h2>
            <div style={{ marginBottom: '20px' }}>
              <label style={{ display: 'block', fontSize: '14px', color: '#666', marginBottom: '8px' }}>
                Amount (PC)
              </label>
              <input
                type="number"
                value={withdrawAmount}
                onChange={(e) => setWithdrawAmount(e.target.value)}
                placeholder="Enter amount"
                style={{
                  width: '100%',
                  padding: '12px',
                  border: '1px solid #ddd',
                  borderRadius: '5px',
                  fontSize: '16px'
                }}
              />
            </div>
            <div style={{ fontSize: '14px', color: '#666', marginBottom: '20px' }}>
              Available balance: {user.balance.toLocaleString()} PC
            </div>
            <div style={{ display: 'flex', gap: '10px' }}>
              <button
                style={{
                  flex: 1,
                  padding: '12px',
                  backgroundColor: '#6c757d',
                  color: 'white',
                  border: 'none',
                  borderRadius: '5px',
                  cursor: 'pointer',
                  fontSize: '14px'
                }}
                onClick={() => setShowWithdraw(false)}
              >
                Cancel
              </button>
              <button
                style={{
                  flex: 1,
                  padding: '12px',
                  backgroundColor: '#007bff',
                  color: 'white',
                  border: 'none',
                  borderRadius: '5px',
                  cursor: 'pointer',
                  fontSize: '14px',
                  fontWeight: 'bold'
                }}
                onClick={handleWithdraw}
              >
                Withdraw
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

export default UserCenter
