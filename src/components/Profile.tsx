import React, { useState } from 'react';
import { 
  User, 
  Settings, 
  Shield, 
  Bell, 
  Eye,
  EyeOff,
  Smartphone,
  Mail,
  Lock,
  CreditCard,
  FileText,
  HelpCircle,
  LogOut,
  Edit,
  Camera,
  Check,
  AlertTriangle,
  Globe,
  Moon,
  Sun
} from 'lucide-react';

export const Profile: React.FC = () => {
  const [activeTab, setActiveTab] = useState('profile');
  const [isDarkMode, setIsDarkMode] = useState(true);
  const [notifications, setNotifications] = useState({
    transactions: true,
    bills: true,
    security: true,
    marketing: false
  });

  const userInfo = {
    name: 'Alex Johnson',
    email: 'alex.johnson@email.com',
    phone: '+1 (555) 123-4567',
    joinDate: 'January 2023',
    verificationStatus: 'verified',
    accountLevel: 'Premium'
  };

  const securityFeatures = [
    { name: 'Two-Factor Authentication', enabled: true, description: 'SMS and Authenticator app' },
    { name: 'Biometric Login', enabled: true, description: 'Fingerprint and Face ID' },
    { name: 'Login Alerts', enabled: true, description: 'Email notifications for new logins' },
    { name: 'Session Management', enabled: false, description: 'Auto-logout after inactivity' },
  ];

  const recentActivity = [
    { action: 'Password changed', date: '2025-01-08', device: 'iPhone 15 Pro' },
    { action: 'New device login', date: '2025-01-05', device: 'MacBook Pro' },
    { action: 'Profile updated', date: '2025-01-02', device: 'iPhone 15 Pro' },
    { action: '2FA enabled', date: '2024-12-28', device: 'iPhone 15 Pro' },
  ];

  const tabs = [
    { id: 'profile', label: 'Profile', icon: User },
    { id: 'security', label: 'Security', icon: Shield },
    { id: 'notifications', label: 'Notifications', icon: Bell },
    { id: 'preferences', label: 'Preferences', icon: Settings },
  ];

  return (
    <div className="p-8 space-y-8">
      {/* Header */}
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-bold text-white mb-2">Profile & Settings</h1>
          <p className="text-slate-400">Manage your account and preferences</p>
        </div>
        <button className="bg-gradient-to-r from-purple-500 to-cyan-500 text-white px-6 py-2 rounded-xl font-medium hover:from-purple-600 hover:to-cyan-600 transition-all duration-300 flex items-center gap-2">
          <Edit className="w-4 h-4" />
          Edit Profile
        </button>
      </div>

      {/* Profile Header Card */}
      <div className="bg-gradient-to-r from-purple-600/20 to-cyan-600/20 backdrop-blur-xl rounded-2xl border border-purple-500/30 p-8">
        <div className="flex items-center gap-6">
          <div className="relative">
            <div className="w-24 h-24 bg-gradient-to-r from-purple-500 to-cyan-500 rounded-2xl flex items-center justify-center">
              <User className="w-12 h-12 text-white" />
            </div>
            <button className="absolute -bottom-2 -right-2 w-8 h-8 bg-slate-800 border border-purple-500/30 rounded-lg flex items-center justify-center hover:bg-slate-700 transition-all duration-300">
              <Camera className="w-4 h-4 text-white" />
            </button>
          </div>
          
          <div className="flex-1">
            <div className="flex items-center gap-3 mb-2">
              <h2 className="text-2xl font-bold text-white">{userInfo.name}</h2>
              {userInfo.verificationStatus === 'verified' && (
                <div className="flex items-center gap-1 bg-green-500/20 border border-green-500/30 rounded-lg px-2 py-1">
                  <Check className="w-4 h-4 text-green-400" />
                  <span className="text-green-400 text-sm font-medium">Verified</span>
                </div>
              )}
            </div>
            <p className="text-slate-300 mb-1">{userInfo.email}</p>
            <p className="text-slate-400 text-sm">Member since {userInfo.joinDate}</p>
          </div>
          
          <div className="text-right">
            <div className="bg-gradient-to-r from-orange-500/20 to-yellow-500/20 border border-orange-500/30 rounded-xl px-4 py-2 mb-3">
              <span className="text-orange-400 font-semibold">{userInfo.accountLevel}</span>
            </div>
          </div>
        </div>
      </div>

      {/* Navigation Tabs */}
      <div className="flex space-x-1 bg-slate-800/50 backdrop-blur-xl rounded-xl p-1 border border-purple-500/20">
        {tabs.map((tab) => {
          const Icon = tab.icon;
          return (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`flex items-center gap-2 px-6 py-3 rounded-lg font-medium transition-all duration-300 ${
                activeTab === tab.id
                  ? 'bg-gradient-to-r from-purple-500/20 to-cyan-500/20 text-white border border-purple-500/30'
                  : 'text-slate-400 hover:text-white hover:bg-slate-700/50'
              }`}
            >
              <Icon className="w-4 h-4" />
              {tab.label}
            </button>
          );
        })}
      </div>

      {/* Tab Content */}
      {activeTab === 'profile' && (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          {/* Personal Information */}
          <div className="bg-slate-800/50 backdrop-blur-xl rounded-2xl border border-purple-500/20 p-6">
            <h3 className="text-xl font-semibold text-white mb-6">Personal Information</h3>
            
            <div className="space-y-4">
              <div>
                <label className="block text-slate-400 text-sm mb-2">Full Name</label>
                <input 
                  type="text" 
                  value={userInfo.name}
                  className="w-full bg-slate-700/50 border border-slate-600/30 rounded-xl px-4 py-3 text-white focus:outline-none focus:border-purple-500/50"
                />
              </div>
              
              <div>
                <label className="block text-slate-400 text-sm mb-2">Email Address</label>
                <input 
                  type="email" 
                  value={userInfo.email}
                  className="w-full bg-slate-700/50 border border-slate-600/30 rounded-xl px-4 py-3 text-white focus:outline-none focus:border-purple-500/50"
                />
              </div>
              
              <div>
                <label className="block text-slate-400 text-sm mb-2">Phone Number</label>
                <input 
                  type="tel" 
                  value={userInfo.phone}
                  className="w-full bg-slate-700/50 border border-slate-600/30 rounded-xl px-4 py-3 text-white focus:outline-none focus:border-purple-500/50"
                />
              </div>
              
              <button className="w-full bg-gradient-to-r from-purple-500 to-cyan-500 text-white py-3 rounded-xl font-medium hover:from-purple-600 hover:to-cyan-600 transition-all duration-300">
                Save Changes
              </button>
            </div>
          </div>

          {/* Account Status */}
          <div className="bg-slate-800/50 backdrop-blur-xl rounded-2xl border border-purple-500/20 p-6">
            <h3 className="text-xl font-semibold text-white mb-6">Account Status</h3>
            
            <div className="space-y-6">
              <div className="flex items-center justify-between p-4 bg-green-500/10 border border-green-500/30 rounded-xl">
                <div className="flex items-center gap-3">
                  <Check className="w-5 h-5 text-green-400" />
                  <div>
                    <p className="text-white font-medium">Identity Verified</p>
                    <p className="text-slate-400 text-sm">Full KYC completed</p>
                  </div>
                </div>
              </div>
              
              <div className="flex items-center justify-between p-4 bg-green-500/10 border border-green-500/30 rounded-xl">
                <div className="flex items-center gap-3">
                  <Check className="w-5 h-5 text-green-400" />
                  <div>
                    <p className="text-white font-medium">Email Verified</p>
                    <p className="text-slate-400 text-sm">Confirmed on Jan 2, 2023</p>
                  </div>
                </div>
              </div>
              
              <div className="flex items-center justify-between p-4 bg-yellow-500/10 border border-yellow-500/30 rounded-xl">
                <div className="flex items-center gap-3">
                  <AlertTriangle className="w-5 h-5 text-yellow-400" />
                  <div>
                    <p className="text-white font-medium">Phone Verification</p>
                    <p className="text-slate-400 text-sm">Verification pending</p>
                  </div>
                </div>
                <button className="text-cyan-400 hover:text-cyan-300 font-medium text-sm">
                  Verify Now
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {activeTab === 'security' && (
        <div className="space-y-6">
          {/* Security Features */}
          <div className="bg-slate-800/50 backdrop-blur-xl rounded-2xl border border-purple-500/20 p-6">
            <h3 className="text-xl font-semibold text-white mb-6">Security Features</h3>
            
            <div className="space-y-4">
              {securityFeatures.map((feature, index) => (
                <div key={index} className="flex items-center justify-between p-4 bg-slate-700/30 rounded-xl">
                  <div>
                    <p className="text-white font-medium">{feature.name}</p>
                    <p className="text-slate-400 text-sm">{feature.description}</p>
                  </div>
                  <label className="relative inline-flex items-center cursor-pointer">
                    <input type="checkbox" checked={feature.enabled} className="sr-only peer" />
                    <div className="w-11 h-6 bg-slate-600 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-gradient-to-r peer-checked:from-purple-500 peer-checked:to-cyan-500"></div>
                  </label>
                </div>
              ))}
            </div>
          </div>

          {/* Recent Activity */}
          <div className="bg-slate-800/50 backdrop-blur-xl rounded-2xl border border-purple-500/20 p-6">
            <h3 className="text-xl font-semibold text-white mb-6">Recent Security Activity</h3>
            
            <div className="space-y-4">
              {recentActivity.map((activity, index) => (
                <div key={index} className="flex items-center gap-4 p-3 hover:bg-slate-700/30 rounded-xl transition-all duration-300">
                  <div className="w-10 h-10 bg-slate-700/50 rounded-xl flex items-center justify-center">
                    <Shield className="w-5 h-5 text-cyan-400" />
                  </div>
                  <div className="flex-1">
                    <p className="text-white font-medium">{activity.action}</p>
                    <p className="text-slate-400 text-sm">{activity.device}</p>
                  </div>
                  <p className="text-slate-400 text-sm">{activity.date}</p>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}

      {activeTab === 'notifications' && (
        <div className="bg-slate-800/50 backdrop-blur-xl rounded-2xl border border-purple-500/20 p-6">
          <h3 className="text-xl font-semibold text-white mb-6">Notification Preferences</h3>
          
          <div className="space-y-6">
            {Object.entries(notifications).map(([key, enabled]) => (
              <div key={key} className="flex items-center justify-between p-4 bg-slate-700/30 rounded-xl">
                <div>
                  <p className="text-white font-medium capitalize">{key} Notifications</p>
                  <p className="text-slate-400 text-sm">
                    {key === 'transactions' && 'Get notified about payments and transfers'}
                    {key === 'bills' && 'Reminders for upcoming bill payments'}
                    {key === 'security' && 'Security alerts and login notifications'}
                    {key === 'marketing' && 'Promotional offers and updates'}
                  </p>
                </div>
                <label className="relative inline-flex items-center cursor-pointer">
                  <input 
                    type="checkbox" 
                    checked={enabled} 
                    onChange={() => setNotifications(prev => ({ ...prev, [key]: !enabled }))}
                    className="sr-only peer" 
                  />
                  <div className="w-11 h-6 bg-slate-600 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-gradient-to-r peer-checked:from-purple-500 peer-checked:to-cyan-500"></div>
                </label>
              </div>
            ))}
          </div>
        </div>
      )}

      {activeTab === 'preferences' && (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <div className="bg-slate-800/50 backdrop-blur-xl rounded-2xl border border-purple-500/20 p-6">
            <h3 className="text-xl font-semibold text-white mb-6">App Preferences</h3>
            
            <div className="space-y-4">
              <div className="flex items-center justify-between p-4 bg-slate-700/30 rounded-xl">
                <div className="flex items-center gap-3">
                  {isDarkMode ? <Moon className="w-5 h-5 text-slate-400" /> : <Sun className="w-5 h-5 text-yellow-400" />}
                  <div>
                    <p className="text-white font-medium">Dark Mode</p>
                    <p className="text-slate-400 text-sm">Use dark theme</p>
                  </div>
                </div>
                <label className="relative inline-flex items-center cursor-pointer">
                  <input 
                    type="checkbox" 
                    checked={isDarkMode} 
                    onChange={() => setIsDarkMode(!isDarkMode)}
                    className="sr-only peer" 
                  />
                  <div className="w-11 h-6 bg-slate-600 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-gradient-to-r peer-checked:from-purple-500 peer-checked:to-cyan-500"></div>
                </label>
              </div>

              <div className="p-4 bg-slate-700/30 rounded-xl">
                <div className="flex items-center gap-3 mb-3">
                  <Globe className="w-5 h-5 text-slate-400" />
                  <p className="text-white font-medium">Language</p>
                </div>
                <select className="w-full bg-slate-600/50 border border-slate-500/30 rounded-lg px-3 py-2 text-white focus:outline-none focus:border-purple-500/50">
                  <option>English (US)</option>
                  <option>Spanish</option>
                  <option>French</option>
                  <option>German</option>
                </select>
              </div>

              <div className="p-4 bg-slate-700/30 rounded-xl">
                <div className="flex items-center gap-3 mb-3">
                  <CreditCard className="w-5 h-5 text-slate-400" />
                  <p className="text-white font-medium">Currency</p>
                </div>
                <select className="w-full bg-slate-600/50 border border-slate-500/30 rounded-lg px-3 py-2 text-white focus:outline-none focus:border-purple-500/50">
                  <option>USD ($)</option>
                  <option>EUR (€)</option>
                  <option>GBP (£)</option>
                  <option>CAD (C$)</option>
                </select>
              </div>
            </div>
          </div>

          <div className="bg-slate-800/50 backdrop-blur-xl rounded-2xl border border-purple-500/20 p-6">
            <h3 className="text-xl font-semibold text-white mb-6">Support & Help</h3>
            
            <div className="space-y-3">
              <button className="w-full flex items-center gap-3 p-4 bg-slate-700/30 rounded-xl hover:bg-slate-600/50 transition-all duration-300">
                <HelpCircle className="w-5 h-5 text-slate-400" />
                <span className="text-white font-medium">Help Center</span>
              </button>
              
              <button className="w-full flex items-center gap-3 p-4 bg-slate-700/30 rounded-xl hover:bg-slate-600/50 transition-all duration-300">
                <FileText className="w-5 h-5 text-slate-400" />
                <span className="text-white font-medium">Terms & Conditions</span>
              </button>
              
              <button className="w-full flex items-center gap-3 p-4 bg-slate-700/30 rounded-xl hover:bg-slate-600/50 transition-all duration-300">
                <Shield className="w-5 h-5 text-slate-400" />
                <span className="text-white font-medium">Privacy Policy</span>
              </button>
              
              <button className="w-full flex items-center gap-3 p-4 bg-red-500/10 border border-red-500/30 rounded-xl hover:bg-red-500/20 transition-all duration-300 text-red-400">
                <LogOut className="w-5 h-5" />
                <span className="font-medium">Sign Out</span>
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};