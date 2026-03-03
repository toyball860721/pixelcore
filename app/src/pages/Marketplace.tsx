import { useState, useEffect } from 'react'

interface AgentListing {
  id: string
  name: string
  description: string
  version: string
  owner: string
  category: string
  price: number
  priceModel: 'per_call' | 'per_hour' | 'subscription'
  rating: number
  totalTransactions: number
  capabilities: string[]
  status: 'active' | 'inactive'
}

interface Transaction {
  id: string
  agentName: string
  amount: number
  status: 'pending' | 'completed' | 'failed'
  createdAt: string
}

function Marketplace() {
  const [view, setView] = useState<'browse' | 'transactions'>('browse')
  const [searchQuery, setSearchQuery] = useState('')
  const [selectedCategory, setSelectedCategory] = useState<string>('all')
  const [priceFilter, setPriceFilter] = useState<string>('all')
  const [sortBy, setSortBy] = useState<'rating' | 'price' | 'transactions'>('rating')

  // Mock data - in production, this would come from the backend
  const [agents] = useState<AgentListing[]>([
    {
      id: '1',
      name: 'Data Analyzer Pro',
      description: 'Advanced data analysis and visualization agent with ML capabilities',
      version: '2.1.0',
      owner: 'DataCorp',
      category: 'analytics',
      price: 50,
      priceModel: 'per_call',
      rating: 4.8,
      totalTransactions: 1250,
      capabilities: ['Data Analysis', 'Visualization', 'ML Predictions'],
      status: 'active'
    },
    {
      id: '2',
      name: 'Content Writer AI',
      description: 'Professional content generation for blogs, articles, and marketing',
      version: '1.5.2',
      owner: 'WriteBot Inc',
      category: 'content',
      price: 99,
      priceModel: 'subscription',
      rating: 4.6,
      totalTransactions: 890,
      capabilities: ['Blog Writing', 'SEO Optimization', 'Social Media'],
      status: 'active'
    },
    {
      id: '3',
      name: 'Code Review Assistant',
      description: 'Automated code review with security and performance analysis',
      version: '3.0.1',
      owner: 'DevTools Ltd',
      category: 'development',
      price: 25,
      priceModel: 'per_call',
      rating: 4.9,
      totalTransactions: 2100,
      capabilities: ['Code Review', 'Security Scan', 'Performance Analysis'],
      status: 'active'
    },
    {
      id: '4',
      name: 'Customer Support Bot',
      description: '24/7 customer support with multi-language capabilities',
      version: '2.3.0',
      owner: 'SupportAI',
      category: 'support',
      price: 15,
      priceModel: 'per_hour',
      rating: 4.5,
      totalTransactions: 3500,
      capabilities: ['Multi-language', 'Ticket Management', 'FAQ Automation'],
      status: 'active'
    },
    {
      id: '5',
      name: 'Image Generator Pro',
      description: 'High-quality image generation for creative projects',
      version: '1.8.0',
      owner: 'CreativeAI',
      category: 'creative',
      price: 199,
      priceModel: 'subscription',
      rating: 4.7,
      totalTransactions: 650,
      capabilities: ['Image Generation', 'Style Transfer', 'Upscaling'],
      status: 'active'
    }
  ])

  const [transactions] = useState<Transaction[]>([
    { id: 't1', agentName: 'Data Analyzer Pro', amount: 50, status: 'completed', createdAt: '2026-03-03T10:30:00Z' },
    { id: 't2', agentName: 'Code Review Assistant', amount: 25, status: 'completed', createdAt: '2026-03-02T15:20:00Z' },
    { id: 't3', agentName: 'Content Writer AI', amount: 99, status: 'pending', createdAt: '2026-03-01T09:15:00Z' }
  ])

  const categories = ['all', 'analytics', 'content', 'development', 'support', 'creative']

  const filteredAgents = agents
    .filter(agent => {
      const matchesSearch = agent.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
                          agent.description.toLowerCase().includes(searchQuery.toLowerCase())
      const matchesCategory = selectedCategory === 'all' || agent.category === selectedCategory
      const matchesPrice = priceFilter === 'all' ||
                          (priceFilter === 'free' && agent.price === 0) ||
                          (priceFilter === 'low' && agent.price > 0 && agent.price <= 50) ||
                          (priceFilter === 'medium' && agent.price > 50 && agent.price <= 150) ||
                          (priceFilter === 'high' && agent.price > 150)
      return matchesSearch && matchesCategory && matchesPrice
    })
    .sort((a, b) => {
      if (sortBy === 'rating') return b.rating - a.rating
      if (sortBy === 'price') return a.price - b.price
      if (sortBy === 'transactions') return b.totalTransactions - a.totalTransactions
      return 0
    })

  const getPriceDisplay = (agent: AgentListing) => {
    if (agent.price === 0) return 'Free'
    const models = {
      per_call: '/call',
      per_hour: '/hour',
      subscription: '/month'
    }
    return `${agent.price} PC${models[agent.priceModel]}`
  }

  return (
    <div style={{ padding: '20px', maxWidth: '1400px', margin: '0 auto' }}>
      {/* Header */}
      <div style={{ marginBottom: '30px' }}>
        <h1 style={{ fontSize: '32px', marginBottom: '10px' }}>🏪 Agent Marketplace</h1>
        <p style={{ color: '#666', fontSize: '16px' }}>
          Discover and purchase AI agents for your business needs
        </p>
      </div>

      {/* View Toggle */}
      <div style={{ display: 'flex', gap: '10px', marginBottom: '20px' }}>
        <button
          onClick={() => setView('browse')}
          style={{
            padding: '10px 20px',
            backgroundColor: view === 'browse' ? '#007bff' : '#f0f0f0',
            color: view === 'browse' ? 'white' : '#333',
            border: 'none',
            borderRadius: '5px',
            cursor: 'pointer',
            fontSize: '14px',
            fontWeight: view === 'browse' ? 'bold' : 'normal'
          }}
        >
          Browse Agents
        </button>
        <button
          onClick={() => setView('transactions')}
          style={{
            padding: '10px 20px',
            backgroundColor: view === 'transactions' ? '#007bff' : '#f0f0f0',
            color: view === 'transactions' ? 'white' : '#333',
            border: 'none',
            borderRadius: '5px',
            cursor: 'pointer',
            fontSize: '14px',
            fontWeight: view === 'transactions' ? 'bold' : 'normal'
          }}
        >
          My Transactions ({transactions.length})
        </button>
      </div>

      {view === 'browse' ? (
        <>
          {/* Search and Filters */}
          <div style={{
            backgroundColor: '#f8f9fa',
            padding: '20px',
            borderRadius: '8px',
            marginBottom: '20px'
          }}>
            <div style={{ display: 'flex', gap: '15px', flexWrap: 'wrap', alignItems: 'center' }}>
              {/* Search */}
              <input
                type="text"
                placeholder="Search agents..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                style={{
                  flex: '1',
                  minWidth: '250px',
                  padding: '10px 15px',
                  border: '1px solid #ddd',
                  borderRadius: '5px',
                  fontSize: '14px'
                }}
              />

              {/* Category Filter */}
              <select
                value={selectedCategory}
                onChange={(e) => setSelectedCategory(e.target.value)}
                style={{
                  padding: '10px 15px',
                  border: '1px solid #ddd',
                  borderRadius: '5px',
                  fontSize: '14px',
                  backgroundColor: 'white'
                }}
              >
                {categories.map(cat => (
                  <option key={cat} value={cat}>
                    {cat.charAt(0).toUpperCase() + cat.slice(1)}
                  </option>
                ))}
              </select>

              {/* Price Filter */}
              <select
                value={priceFilter}
                onChange={(e) => setPriceFilter(e.target.value)}
                style={{
                  padding: '10px 15px',
                  border: '1px solid #ddd',
                  borderRadius: '5px',
                  fontSize: '14px',
                  backgroundColor: 'white'
                }}
              >
                <option value="all">All Prices</option>
                <option value="free">Free</option>
                <option value="low">Low (≤50 PC)</option>
                <option value="medium">Medium (51-150 PC)</option>
                <option value="high">High (&gt;150 PC)</option>
              </select>

              {/* Sort */}
              <select
                value={sortBy}
                onChange={(e) => setSortBy(e.target.value as any)}
                style={{
                  padding: '10px 15px',
                  border: '1px solid #ddd',
                  borderRadius: '5px',
                  fontSize: '14px',
                  backgroundColor: 'white'
                }}
              >
                <option value="rating">Sort by Rating</option>
                <option value="price">Sort by Price</option>
                <option value="transactions">Sort by Popularity</option>
              </select>
            </div>
          </div>

          {/* Results Count */}
          <div style={{ marginBottom: '15px', color: '#666' }}>
            Found {filteredAgents.length} agent{filteredAgents.length !== 1 ? 's' : ''}
          </div>

          {/* Agent Grid */}
          <div style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fill, minmax(350px, 1fr))',
            gap: '20px'
          }}>
            {filteredAgents.map(agent => (
              <div
                key={agent.id}
                style={{
                  backgroundColor: 'white',
                  border: '1px solid #e0e0e0',
                  borderRadius: '8px',
                  padding: '20px',
                  cursor: 'pointer',
                  transition: 'all 0.2s',
                  boxShadow: '0 2px 4px rgba(0,0,0,0.05)'
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.boxShadow = '0 4px 12px rgba(0,0,0,0.1)'
                  e.currentTarget.style.transform = 'translateY(-2px)'
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.boxShadow = '0 2px 4px rgba(0,0,0,0.05)'
                  e.currentTarget.style.transform = 'translateY(0)'
                }}
              >
                {/* Header */}
                <div style={{ marginBottom: '12px' }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'start', marginBottom: '8px' }}>
                    <h3 style={{ fontSize: '18px', margin: 0 }}>{agent.name}</h3>
                    <span style={{
                      backgroundColor: '#28a745',
                      color: 'white',
                      padding: '2px 8px',
                      borderRadius: '12px',
                      fontSize: '11px',
                      fontWeight: 'bold'
                    }}>
                      {agent.status.toUpperCase()}
                    </span>
                  </div>
                  <div style={{ fontSize: '12px', color: '#666' }}>
                    by {agent.owner} • v{agent.version}
                  </div>
                </div>

                {/* Description */}
                <p style={{ fontSize: '14px', color: '#555', marginBottom: '15px', lineHeight: '1.5' }}>
                  {agent.description}
                </p>

                {/* Capabilities */}
                <div style={{ marginBottom: '15px' }}>
                  <div style={{ display: 'flex', flexWrap: 'wrap', gap: '6px' }}>
                    {agent.capabilities.map((cap, idx) => (
                      <span
                        key={idx}
                        style={{
                          backgroundColor: '#e3f2fd',
                          color: '#1976d2',
                          padding: '4px 10px',
                          borderRadius: '4px',
                          fontSize: '12px'
                        }}
                      >
                        {cap}
                      </span>
                    ))}
                  </div>
                </div>

                {/* Stats */}
                <div style={{
                  display: 'flex',
                  justifyContent: 'space-between',
                  alignItems: 'center',
                  paddingTop: '15px',
                  borderTop: '1px solid #f0f0f0'
                }}>
                  <div style={{ display: 'flex', gap: '15px', fontSize: '13px', color: '#666' }}>
                    <span>⭐ {agent.rating.toFixed(1)}</span>
                    <span>📊 {agent.totalTransactions.toLocaleString()} uses</span>
                  </div>
                  <div style={{ fontSize: '16px', fontWeight: 'bold', color: '#007bff' }}>
                    {getPriceDisplay(agent)}
                  </div>
                </div>

                {/* Action Button */}
                <button
                  style={{
                    width: '100%',
                    marginTop: '15px',
                    padding: '10px',
                    backgroundColor: '#007bff',
                    color: 'white',
                    border: 'none',
                    borderRadius: '5px',
                    cursor: 'pointer',
                    fontSize: '14px',
                    fontWeight: 'bold'
                  }}
                  onClick={() => alert(`Purchasing ${agent.name}...`)}
                >
                  Purchase Agent
                </button>
              </div>
            ))}
          </div>

          {filteredAgents.length === 0 && (
            <div style={{
              textAlign: 'center',
              padding: '60px 20px',
              color: '#999'
            }}>
              <h3>No agents found</h3>
              <p>Try adjusting your search or filters</p>
            </div>
          )}
        </>
      ) : (
        /* Transactions View */
        <div>
          <h2 style={{ marginBottom: '20px' }}>Transaction History</h2>
          <div style={{ backgroundColor: 'white', borderRadius: '8px', overflow: 'hidden', border: '1px solid #e0e0e0' }}>
            <table style={{ width: '100%', borderCollapse: 'collapse' }}>
              <thead>
                <tr style={{ backgroundColor: '#f8f9fa' }}>
                  <th style={{ padding: '15px', textAlign: 'left', borderBottom: '2px solid #dee2e6' }}>Transaction ID</th>
                  <th style={{ padding: '15px', textAlign: 'left', borderBottom: '2px solid #dee2e6' }}>Agent</th>
                  <th style={{ padding: '15px', textAlign: 'left', borderBottom: '2px solid #dee2e6' }}>Amount</th>
                  <th style={{ padding: '15px', textAlign: 'left', borderBottom: '2px solid #dee2e6' }}>Status</th>
                  <th style={{ padding: '15px', textAlign: 'left', borderBottom: '2px solid #dee2e6' }}>Date</th>
                </tr>
              </thead>
              <tbody>
                {transactions.map(tx => (
                  <tr key={tx.id} style={{ borderBottom: '1px solid #f0f0f0' }}>
                    <td style={{ padding: '15px' }}>{tx.id}</td>
                    <td style={{ padding: '15px' }}>{tx.agentName}</td>
                    <td style={{ padding: '15px', fontWeight: 'bold' }}>{tx.amount} PC</td>
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
    </div>
  )
}

export default Marketplace
