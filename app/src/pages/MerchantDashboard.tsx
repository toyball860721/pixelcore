import { useState } from 'react'

interface Service {
  id: string
  name: string
  description: string
  category: string
  price: number
  priceModel: 'per_call' | 'per_hour' | 'subscription'
  status: 'active' | 'inactive' | 'pending'
  totalOrders: number
  revenue: number
  rating: number
}

interface Order {
  id: string
  serviceName: string
  buyer: string
  amount: number
  status: 'pending' | 'processing' | 'completed' | 'cancelled'
  createdAt: string
}

interface RevenueData {
  month: string
  revenue: number
}

function MerchantDashboard() {
  const [activeView, setActiveView] = useState<'overview' | 'services' | 'orders'>('overview')
  const [showAddService, setShowAddService] = useState(false)

  // Mock data
  const [services] = useState<Service[]>([
    {
      id: 's1',
      name: 'Data Analyzer Pro',
      description: 'Advanced data analysis and visualization',
      category: 'analytics',
      price: 50,
      priceModel: 'per_call',
      status: 'active',
      totalOrders: 1250,
      revenue: 62500,
      rating: 4.8
    },
    {
      id: 's2',
      name: 'Report Generator',
      description: 'Automated report generation',
      category: 'analytics',
      price: 30,
      priceModel: 'per_call',
      status: 'active',
      totalOrders: 890,
      revenue: 26700,
      rating: 4.6
    },
    {
      id: 's3',
      name: 'ML Predictor',
      description: 'Machine learning predictions',
      category: 'analytics',
      price: 199,
      priceModel: 'subscription',
      status: 'pending',
      totalOrders: 45,
      revenue: 8955,
      rating: 4.9
    }
  ])

  const [orders] = useState<Order[]>([
    { id: 'o1', serviceName: 'Data Analyzer Pro', buyer: 'user@example.com', amount: 50, status: 'completed', createdAt: '2026-03-03T10:30:00Z' },
    { id: 'o2', serviceName: 'Data Analyzer Pro', buyer: 'client@company.com', amount: 50, status: 'processing', createdAt: '2026-03-03T09:15:00Z' },
    { id: 'o3', serviceName: 'Report Generator', buyer: 'admin@business.com', amount: 30, status: 'pending', createdAt: '2026-03-03T08:00:00Z' },
    { id: 'o4', serviceName: 'ML Predictor', buyer: 'data@startup.com', amount: 199, status: 'completed', createdAt: '2026-03-02T16:45:00Z' },
    { id: 'o5', serviceName: 'Report Generator', buyer: 'manager@corp.com', amount: 30, status: 'cancelled', createdAt: '2026-03-02T14:20:00Z' }
  ])

  const [revenueData] = useState<RevenueData[]>([
    { month: 'Oct', revenue: 45000 },
    { month: 'Nov', revenue: 52000 },
    { month: 'Dec', revenue: 48000 },
    { month: 'Jan', revenue: 61000 },
    { month: 'Feb', revenue: 58000 },
    { month: 'Mar', revenue: 98155 }
  ])

  const totalRevenue = services.reduce((sum, s) => sum + s.revenue, 0)
  const totalOrders = services.reduce((sum, s) => sum + s.totalOrders, 0)
  const averageRating = services.reduce((sum, s) => sum + s.rating, 0) / services.length
  const activeServices = services.filter(s => s.status === 'active').length

  const pendingOrders = orders.filter(o => o.status === 'pending').length
  const processingOrders = orders.filter(o => o.status === 'processing').length

  return (
    <div style={{ padding: '20px', maxWidth: '1400px', margin: '0 auto' }}>
      {/* Header */}
      <div style={{ marginBottom: '30px' }}>
        <h1 style={{ fontSize: '32px', marginBottom: '10px' }}>📊 Merchant Dashboard</h1>
        <p style={{ color: '#666', fontSize: '16px' }}>
          Manage your services, orders, and track revenue
        </p>
      </div>

      {/* View Toggle */}
      <div style={{ display: 'flex', gap: '10px', marginBottom: '20px' }}>
        <button
          onClick={() => setActiveView('overview')}
          style={{
            padding: '10px 20px',
            backgroundColor: activeView === 'overview' ? '#007bff' : '#f0f0f0',
            color: activeView === 'overview' ? 'white' : '#333',
            border: 'none',
            borderRadius: '5px',
            cursor: 'pointer',
            fontSize: '14px',
            fontWeight: activeView === 'overview' ? 'bold' : 'normal'
          }}
        >
          Overview
        </button>
        <button
          onClick={() => setActiveView('services')}
          style={{
            padding: '10px 20px',
            backgroundColor: activeView === 'services' ? '#007bff' : '#f0f0f0',
            color: activeView === 'services' ? 'white' : '#333',
            border: 'none',
            borderRadius: '5px',
            cursor: 'pointer',
            fontSize: '14px',
            fontWeight: activeView === 'services' ? 'bold' : 'normal'
          }}
        >
          My Services ({services.length})
        </button>
        <button
          onClick={() => setActiveView('orders')}
          style={{
            padding: '10px 20px',
            backgroundColor: activeView === 'orders' ? '#007bff' : '#f0f0f0',
            color: activeView === 'orders' ? 'white' : '#333',
            border: 'none',
            borderRadius: '5px',
            cursor: 'pointer',
            fontSize: '14px',
            fontWeight: activeView === 'orders' ? 'bold' : 'normal'
          }}
        >
          Orders ({orders.length})
          {pendingOrders > 0 && (
            <span style={{
              marginLeft: '5px',
              backgroundColor: '#dc3545',
              color: 'white',
              padding: '2px 6px',
              borderRadius: '10px',
              fontSize: '11px'
            }}>
              {pendingOrders}
            </span>
          )}
        </button>
      </div>

      {/* Overview View */}
      {activeView === 'overview' && (
        <div>
          {/* Stats Cards */}
          <div style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fit, minmax(250px, 1fr))',
            gap: '20px',
            marginBottom: '30px'
          }}>
            <div style={{
              backgroundColor: 'white',
              padding: '20px',
              borderRadius: '8px',
              border: '1px solid #e0e0e0',
              boxShadow: '0 2px 4px rgba(0,0,0,0.05)'
            }}>
              <div style={{ fontSize: '14px', color: '#666', marginBottom: '8px' }}>Total Revenue</div>
              <div style={{ fontSize: '28px', fontWeight: 'bold', color: '#28a745' }}>
                {totalRevenue.toLocaleString()} PC
              </div>
              <div style={{ fontSize: '12px', color: '#28a745', marginTop: '5px' }}>
                ↑ +15.2% from last month
              </div>
            </div>

            <div style={{
              backgroundColor: 'white',
              padding: '20px',
              borderRadius: '8px',
              border: '1px solid #e0e0e0',
              boxShadow: '0 2px 4px rgba(0,0,0,0.05)'
            }}>
              <div style={{ fontSize: '14px', color: '#666', marginBottom: '8px' }}>Total Orders</div>
              <div style={{ fontSize: '28px', fontWeight: 'bold', color: '#007bff' }}>
                {totalOrders.toLocaleString()}
              </div>
              <div style={{ fontSize: '12px', color: '#007bff', marginTop: '5px' }}>
                {pendingOrders} pending, {processingOrders} processing
              </div>
            </div>

            <div style={{
              backgroundColor: 'white',
              padding: '20px',
              borderRadius: '8px',
              border: '1px solid #e0e0e0',
              boxShadow: '0 2px 4px rgba(0,0,0,0.05)'
            }}>
              <div style={{ fontSize: '14px', color: '#666', marginBottom: '8px' }}>Active Services</div>
              <div style={{ fontSize: '28px', fontWeight: 'bold', color: '#6f42c1' }}>
                {activeServices} / {services.length}
              </div>
              <div style={{ fontSize: '12px', color: '#666', marginTop: '5px' }}>
                {services.length - activeServices} inactive
              </div>
            </div>

            <div style={{
              backgroundColor: 'white',
              padding: '20px',
              borderRadius: '8px',
              border: '1px solid #e0e0e0',
              boxShadow: '0 2px 4px rgba(0,0,0,0.05)'
            }}>
              <div style={{ fontSize: '14px', color: '#666', marginBottom: '8px' }}>Average Rating</div>
              <div style={{ fontSize: '28px', fontWeight: 'bold', color: '#ffc107' }}>
                ⭐ {averageRating.toFixed(1)}
              </div>
              <div style={{ fontSize: '12px', color: '#666', marginTop: '5px' }}>
                Based on {totalOrders} reviews
              </div>
            </div>
          </div>

          {/* Revenue Chart */}
          <div style={{
            backgroundColor: 'white',
            padding: '20px',
            borderRadius: '8px',
            border: '1px solid #e0e0e0',
            marginBottom: '30px'
          }}>
            <h3 style={{ marginBottom: '20px' }}>Revenue Trend (Last 6 Months)</h3>
            <div style={{ display: 'flex', alignItems: 'flex-end', gap: '15px', height: '200px' }}>
              {revenueData.map((data, idx) => {
                const maxRevenue = Math.max(...revenueData.map(d => d.revenue))
                const height = (data.revenue / maxRevenue) * 180
                return (
                  <div key={idx} style={{ flex: 1, display: 'flex', flexDirection: 'column', alignItems: 'center' }}>
                    <div style={{ fontSize: '12px', color: '#666', marginBottom: '5px' }}>
                      {(data.revenue / 1000).toFixed(0)}k
                    </div>
                    <div
                      style={{
                        width: '100%',
                        height: `${height}px`,
                        backgroundColor: idx === revenueData.length - 1 ? '#007bff' : '#e3f2fd',
                        borderRadius: '4px 4px 0 0',
                        transition: 'all 0.3s'
                      }}
                    />
                    <div style={{ fontSize: '12px', color: '#666', marginTop: '5px', fontWeight: 'bold' }}>
                      {data.month}
                    </div>
                  </div>
                )
              })}
            </div>
          </div>

          {/* Top Services */}
          <div style={{
            backgroundColor: 'white',
            padding: '20px',
            borderRadius: '8px',
            border: '1px solid #e0e0e0'
          }}>
            <h3 style={{ marginBottom: '15px' }}>Top Performing Services</h3>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
              {services
                .sort((a, b) => b.revenue - a.revenue)
                .slice(0, 3)
                .map((service, idx) => (
                  <div
                    key={service.id}
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
                      <div style={{
                        width: '30px',
                        height: '30px',
                        borderRadius: '50%',
                        backgroundColor: idx === 0 ? '#ffd700' : idx === 1 ? '#c0c0c0' : '#cd7f32',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        fontWeight: 'bold',
                        color: 'white'
                      }}>
                        {idx + 1}
                      </div>
                      <div>
                        <div style={{ fontWeight: 'bold' }}>{service.name}</div>
                        <div style={{ fontSize: '12px', color: '#666' }}>
                          {service.totalOrders} orders • ⭐ {service.rating}
                        </div>
                      </div>
                    </div>
                    <div style={{ fontSize: '18px', fontWeight: 'bold', color: '#28a745' }}>
                      {service.revenue.toLocaleString()} PC
                    </div>
                  </div>
                ))}
            </div>
          </div>
        </div>
      )}

      {/* Services View */}
      {activeView === 'services' && (
        <div>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '20px' }}>
            <h2>My Services</h2>
            <button
              onClick={() => setShowAddService(true)}
              style={{
                padding: '10px 20px',
                backgroundColor: '#28a745',
                color: 'white',
                border: 'none',
                borderRadius: '5px',
                cursor: 'pointer',
                fontSize: '14px',
                fontWeight: 'bold'
              }}
            >
              + Add New Service
            </button>
          </div>

          <div style={{ display: 'flex', flexDirection: 'column', gap: '15px' }}>
            {services.map(service => (
              <div
                key={service.id}
                style={{
                  backgroundColor: 'white',
                  border: '1px solid #e0e0e0',
                  borderRadius: '8px',
                  padding: '20px'
                }}
              >
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'start' }}>
                  <div style={{ flex: 1 }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '10px', marginBottom: '8px' }}>
                      <h3 style={{ margin: 0 }}>{service.name}</h3>
                      <span style={{
                        padding: '3px 10px',
                        borderRadius: '12px',
                        fontSize: '11px',
                        fontWeight: 'bold',
                        backgroundColor: service.status === 'active' ? '#d4edda' : service.status === 'pending' ? '#fff3cd' : '#f8d7da',
                        color: service.status === 'active' ? '#155724' : service.status === 'pending' ? '#856404' : '#721c24'
                      }}>
                        {service.status.toUpperCase()}
                      </span>
                    </div>
                    <p style={{ color: '#666', marginBottom: '15px' }}>{service.description}</p>
                    <div style={{ display: 'flex', gap: '20px', fontSize: '14px', color: '#666' }}>
                      <span>💰 {service.price} PC/{service.priceModel.replace('_', ' ')}</span>
                      <span>📊 {service.totalOrders} orders</span>
                      <span>⭐ {service.rating}</span>
                      <span>💵 {service.revenue.toLocaleString()} PC revenue</span>
                    </div>
                  </div>
                  <div style={{ display: 'flex', gap: '10px' }}>
                    <button
                      style={{
                        padding: '8px 15px',
                        backgroundColor: '#007bff',
                        color: 'white',
                        border: 'none',
                        borderRadius: '5px',
                        cursor: 'pointer',
                        fontSize: '13px'
                      }}
                      onClick={() => alert(`Editing ${service.name}...`)}
                    >
                      Edit
                    </button>
                    <button
                      style={{
                        padding: '8px 15px',
                        backgroundColor: service.status === 'active' ? '#ffc107' : '#28a745',
                        color: 'white',
                        border: 'none',
                        borderRadius: '5px',
                        cursor: 'pointer',
                        fontSize: '13px'
                      }}
                      onClick={() => alert(`Toggling status for ${service.name}...`)}
                    >
                      {service.status === 'active' ? 'Deactivate' : 'Activate'}
                    </button>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Orders View */}
      {activeView === 'orders' && (
        <div>
          <h2 style={{ marginBottom: '20px' }}>Order Management</h2>
          <div style={{ backgroundColor: 'white', borderRadius: '8px', overflow: 'hidden', border: '1px solid #e0e0e0' }}>
            <table style={{ width: '100%', borderCollapse: 'collapse' }}>
              <thead>
                <tr style={{ backgroundColor: '#f8f9fa' }}>
                  <th style={{ padding: '15px', textAlign: 'left', borderBottom: '2px solid #dee2e6' }}>Order ID</th>
                  <th style={{ padding: '15px', textAlign: 'left', borderBottom: '2px solid #dee2e6' }}>Service</th>
                  <th style={{ padding: '15px', textAlign: 'left', borderBottom: '2px solid #dee2e6' }}>Buyer</th>
                  <th style={{ padding: '15px', textAlign: 'left', borderBottom: '2px solid #dee2e6' }}>Amount</th>
                  <th style={{ padding: '15px', textAlign: 'left', borderBottom: '2px solid #dee2e6' }}>Status</th>
                  <th style={{ padding: '15px', textAlign: 'left', borderBottom: '2px solid #dee2e6' }}>Date</th>
                  <th style={{ padding: '15px', textAlign: 'left', borderBottom: '2px solid #dee2e6' }}>Actions</th>
                </tr>
              </thead>
              <tbody>
                {orders.map(order => (
                  <tr key={order.id} style={{ borderBottom: '1px solid #f0f0f0' }}>
                    <td style={{ padding: '15px' }}>{order.id}</td>
                    <td style={{ padding: '15px' }}>{order.serviceName}</td>
                    <td style={{ padding: '15px' }}>{order.buyer}</td>
                    <td style={{ padding: '15px', fontWeight: 'bold' }}>{order.amount} PC</td>
                    <td style={{ padding: '15px' }}>
                      <span style={{
                        padding: '4px 12px',
                        borderRadius: '12px',
                        fontSize: '12px',
                        fontWeight: 'bold',
                        backgroundColor:
                          order.status === 'completed' ? '#d4edda' :
                          order.status === 'processing' ? '#cfe2ff' :
                          order.status === 'pending' ? '#fff3cd' : '#f8d7da',
                        color:
                          order.status === 'completed' ? '#155724' :
                          order.status === 'processing' ? '#084298' :
                          order.status === 'pending' ? '#856404' : '#721c24'
                      }}>
                        {order.status.toUpperCase()}
                      </span>
                    </td>
                    <td style={{ padding: '15px' }}>{new Date(order.createdAt).toLocaleString()}</td>
                    <td style={{ padding: '15px' }}>
                      {order.status === 'pending' && (
                        <button
                          style={{
                            padding: '6px 12px',
                            backgroundColor: '#28a745',
                            color: 'white',
                            border: 'none',
                            borderRadius: '4px',
                            cursor: 'pointer',
                            fontSize: '12px'
                          }}
                          onClick={() => alert(`Processing order ${order.id}...`)}
                        >
                          Process
                        </button>
                      )}
                      {order.status === 'processing' && (
                        <button
                          style={{
                            padding: '6px 12px',
                            backgroundColor: '#007bff',
                            color: 'white',
                            border: 'none',
                            borderRadius: '4px',
                            cursor: 'pointer',
                            fontSize: '12px'
                          }}
                          onClick={() => alert(`Completing order ${order.id}...`)}
                        >
                          Complete
                        </button>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Add Service Modal */}
      {showAddService && (
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
          onClick={() => setShowAddService(false)}
        >
          <div
            style={{
              backgroundColor: 'white',
              padding: '30px',
              borderRadius: '8px',
              maxWidth: '500px',
              width: '90%'
            }}
            onClick={(e) => e.stopPropagation()}
          >
            <h2 style={{ marginBottom: '20px' }}>Add New Service</h2>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '15px' }}>
              <input type="text" placeholder="Service Name" style={{ padding: '10px', border: '1px solid #ddd', borderRadius: '5px' }} />
              <textarea placeholder="Description" rows={3} style={{ padding: '10px', border: '1px solid #ddd', borderRadius: '5px' }} />
              <select style={{ padding: '10px', border: '1px solid #ddd', borderRadius: '5px' }}>
                <option>Select Category</option>
                <option>Analytics</option>
                <option>Content</option>
                <option>Development</option>
                <option>Support</option>
              </select>
              <input type="number" placeholder="Price (PC)" style={{ padding: '10px', border: '1px solid #ddd', borderRadius: '5px' }} />
              <select style={{ padding: '10px', border: '1px solid #ddd', borderRadius: '5px' }}>
                <option value="per_call">Per Call</option>
                <option value="per_hour">Per Hour</option>
                <option value="subscription">Subscription</option>
              </select>
              <div style={{ display: 'flex', gap: '10px', marginTop: '10px' }}>
                <button
                  style={{
                    flex: 1,
                    padding: '10px',
                    backgroundColor: '#6c757d',
                    color: 'white',
                    border: 'none',
                    borderRadius: '5px',
                    cursor: 'pointer'
                  }}
                  onClick={() => setShowAddService(false)}
                >
                  Cancel
                </button>
                <button
                  style={{
                    flex: 1,
                    padding: '10px',
                    backgroundColor: '#28a745',
                    color: 'white',
                    border: 'none',
                    borderRadius: '5px',
                    cursor: 'pointer'
                  }}
                  onClick={() => {
                    alert('Service added successfully!')
                    setShowAddService(false)
                  }}
                >
                  Add Service
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

export default MerchantDashboard
