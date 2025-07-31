import React, { useState } from 'react';
import { 
  Calendar,
  Clock,
  AlertCircle,
  CheckCircle,
  Plus,
  Filter,
  Search,
  Zap,
  Wifi,
  Phone,
  Car,
  Home,
  Heart,
  CreditCard,
  Repeat,
  DollarSign
} from 'lucide-react';

export const Bills: React.FC = () => {
  const [activeTab, setActiveTab] = useState('upcoming');
  const [searchTerm, setSearchTerm] = useState('');

  const upcomingBills = [
    { id: 1, name: 'Electric Bill', company: 'City Power Co.', amount: 89.50, dueDate: '2025-01-12', status: 'pending', icon: Zap, color: 'yellow' },
    { id: 2, name: 'Internet', company: 'FastNet ISP', amount: 79.99, dueDate: '2025-01-15', status: 'pending', icon: Wifi, color: 'blue' },
    { id: 3, name: 'Phone Bill', company: 'MobileMax', amount: 45.00, dueDate: '2025-01-18', status: 'pending', icon: Phone, color: 'green' },
    { id: 4, name: 'Car Insurance', company: 'AutoSafe', amount: 156.78, dueDate: '2025-01-20', status: 'pending', icon: Car, color: 'purple' },
    { id: 5, name: 'Rent', company: 'Property Management', amount: 1200.00, dueDate: '2025-01-25', status: 'scheduled', icon: Home, color: 'orange' },
  ];

  const subscriptions = [
    { id: 1, name: 'Netflix', amount: 15.99, renewalDate: '2025-01-10', status: 'active', category: 'Entertainment' },
    { id: 2, name: 'Spotify', amount: 9.99, renewalDate: '2025-01-14', status: 'active', category: 'Music' },
    { id: 3, name: 'Adobe Creative', amount: 52.99, renewalDate: '2025-01-22', status: 'active', category: 'Software' },
    { id: 4, name: 'Gym Membership', amount: 29.99, renewalDate: '2025-01-28', status: 'active', category: 'Health' },
    { id: 5, name: 'Cloud Storage', amount: 4.99, renewalDate: '2025-02-01', status: 'active', category: 'Storage' },
  ];

  const billCategories = [
    { name: 'Utilities', count: 3, total: 214.49, icon: Zap, color: 'yellow' },
    { name: 'Insurance', count: 2, total: 287.56, icon: Heart, color: 'red' },
    { name: 'Subscriptions', count: 5, total: 112.95, icon: Repeat, color: 'purple' },
    { name: 'Loans', count: 1, total: 450.00, icon: CreditCard, color: 'blue' },
  ];

  const getColorClasses = (color: string) => {
    const colors: { [key: string]: { bg: string; text: string; border: string; icon: string } } = {
      yellow: { bg: 'bg-yellow-500', text: 'text-yellow-400', border: 'border-yellow-500/30', icon: 'bg-yellow-500/20' },
      blue: { bg: 'bg-blue-500', text: 'text-blue-400', border: 'border-blue-500/30', icon: 'bg-blue-500/20' },
      green: { bg: 'bg-green-500', text: 'text-green-400', border: 'border-green-500/30', icon: 'bg-green-500/20' },
      purple: { bg: 'bg-purple-500', text: 'text-purple-400', border: 'border-purple-500/30', icon: 'bg-purple-500/20' },
      orange: { bg: 'bg-orange-500', text: 'text-orange-400', border: 'border-orange-500/30', icon: 'bg-orange-500/20' },
      red: { bg: 'bg-red-500', text: 'text-red-400', border: 'border-red-500/30', icon: 'bg-red-500/20' },
    };
    return colors[color] || colors.purple;
  };

  const getDaysUntilDue = (dueDate: string) => {
    const today = new Date();
    const due = new Date(dueDate);
    const diffTime = due.getTime() - today.getTime();
    const diffDays = Math.ceil(diffTime / (1000 * 60 * 60 * 24));
    return diffDays;
  };

  return (
    <div className="p-8 space-y-8">
      {/* Header */}
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-bold text-white mb-2">Bills & Subscriptions</h1>
          <p className="text-slate-400">Manage your recurring payments and subscriptions</p>
        </div>
        <button className="bg-gradient-to-r from-purple-500 to-cyan-500 text-white px-6 py-2 rounded-xl font-medium hover:from-purple-600 hover:to-cyan-600 transition-all duration-300 flex items-center gap-2">
          <Plus className="w-4 h-4" />
          Add Bill
        </button>
      </div>

      {/* Overview Cards */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
        {billCategories.map((category, index) => {
          const Icon = category.icon;
          const colors = getColorClasses(category.color);
          
          return (
            <div key={index} className={`bg-slate-800/50 backdrop-blur-xl rounded-2xl border ${colors.border} p-6 hover:bg-slate-700/50 transition-all duration-300`}>
              <div className="flex items-center justify-between mb-4">
                <div className={`w-12 h-12 ${colors.icon} rounded-xl flex items-center justify-center`}>
                  <Icon className={`w-6 h-6 ${colors.text}`} />
                </div>
                <span className={`text-sm ${colors.text} font-medium`}>{category.count} bills</span>
              </div>
              <h3 className="text-white font-semibold mb-1">{category.name}</h3>
              <p className="text-2xl font-bold text-white">${category.total}</p>
            </div>
          );
        })}
      </div>

      {/* Navigation Tabs */}
      <div className="flex space-x-1 bg-slate-800/50 backdrop-blur-xl rounded-xl p-1 border border-purple-500/20">
        {[
          { id: 'upcoming', label: 'Upcoming Bills', icon: Calendar },
          { id: 'subscriptions', label: 'Subscriptions', icon: Repeat },
          { id: 'history', label: 'Payment History', icon: Clock },
        ].map((tab) => {
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

      {/* Search and Filter */}
      <div className="flex items-center gap-4">
        <div className="flex-1 relative">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-5 h-5 text-slate-400" />
          <input
            type="text"
            placeholder="Search bills and subscriptions..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="w-full bg-slate-800/50 border border-purple-500/20 rounded-xl pl-10 pr-4 py-3 text-white placeholder-slate-400 focus:outline-none focus:border-purple-500/50"
          />
        </div>
        <button className="bg-slate-800/50 border border-purple-500/20 rounded-xl px-4 py-3 text-slate-400 hover:text-white hover:bg-slate-700/50 transition-all duration-300 flex items-center gap-2">
          <Filter className="w-4 h-4" />
          Filter
        </button>
      </div>

      {/* Content based on active tab */}
      {activeTab === 'upcoming' && (
        <div className="space-y-4">
          <div className="bg-slate-800/50 backdrop-blur-xl rounded-2xl border border-purple-500/20 p-6">
            <h3 className="text-xl font-semibold text-white mb-6">Upcoming Bills</h3>
            
            <div className="space-y-4">
              {upcomingBills.map((bill) => {
                const Icon = bill.icon;
                const colors = getColorClasses(bill.color);
                const daysUntil = getDaysUntilDue(bill.dueDate);
                const isOverdue = daysUntil < 0;
                const isDueSoon = daysUntil <= 3 && daysUntil >= 0;
                
                return (
                  <div key={bill.id} className={`flex items-center justify-between p-4 rounded-xl border transition-all duration-300 hover:bg-slate-700/30 ${
                    isOverdue ? 'bg-red-500/10 border-red-500/30' : 
                    isDueSoon ? 'bg-yellow-500/10 border-yellow-500/30' : 
                    'bg-slate-700/20 border-slate-600/30'
                  }`}>
                    <div className="flex items-center gap-4">
                      <div className={`w-12 h-12 ${colors.icon} rounded-xl flex items-center justify-center`}>
                        <Icon className={`w-6 h-6 ${colors.text}`} />
                      </div>
                      <div>
                        <h4 className="text-white font-semibold">{bill.name}</h4>
                        <p className="text-slate-400 text-sm">{bill.company}</p>
                      </div>
                    </div>
                    
                    <div className="flex items-center gap-6">
                      <div className="text-right">
                        <p className="text-white font-semibold">${bill.amount}</p>
                        <p className={`text-sm ${
                          isOverdue ? 'text-red-400' : 
                          isDueSoon ? 'text-yellow-400' : 
                          'text-slate-400'
                        }`}>
                          Due {bill.dueDate}
                        </p>
                      </div>
                      
                      <div className="flex items-center gap-2">
                        {isOverdue && <AlertCircle className="w-5 h-5 text-red-400" />}
                        {isDueSoon && <Clock className="w-5 h-5 text-yellow-400" />}
                        {bill.status === 'scheduled' && <CheckCircle className="w-5 h-5 text-green-400" />}
                        
                        <button className="bg-gradient-to-r from-purple-500 to-cyan-500 text-white px-4 py-2 rounded-lg font-medium hover:from-purple-600 hover:to-cyan-600 transition-all duration-300">
                          Pay Now
                        </button>
                      </div>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        </div>
      )}

      {activeTab === 'subscriptions' && (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {subscriptions.map((subscription) => (
            <div key={subscription.id} className="bg-slate-800/50 backdrop-blur-xl rounded-2xl border border-purple-500/20 p-6 hover:bg-slate-700/50 transition-all duration-300">
              <div className="flex items-center justify-between mb-4">
                <div className="w-12 h-12 bg-gradient-to-r from-purple-500/20 to-cyan-500/20 rounded-xl flex items-center justify-center">
                  <Repeat className="w-6 h-6 text-purple-400" />
                </div>
                <span className={`px-2 py-1 rounded-lg text-xs font-medium ${
                  subscription.status === 'active' ? 'bg-green-500/20 text-green-400' : 'bg-red-500/20 text-red-400'
                }`}>
                  {subscription.status}
                </span>
              </div>
              
              <h4 className="text-white font-semibold mb-1">{subscription.name}</h4>
              <p className="text-slate-400 text-sm mb-4">{subscription.category}</p>
              
              <div className="space-y-2 mb-4">
                <div className="flex justify-between">
                  <span className="text-slate-400 text-sm">Amount</span>
                  <span className="text-white font-medium">${subscription.amount}/month</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-slate-400 text-sm">Next Renewal</span>
                  <span className="text-white font-medium">{subscription.renewalDate}</span>
                </div>
              </div>
              
              <div className="flex gap-2">
                <button className="flex-1 bg-slate-700/50 text-white py-2 rounded-lg font-medium hover:bg-slate-600/50 transition-all duration-300">
                  Manage
                </button>
                <button className="flex-1 bg-red-500/20 text-red-400 py-2 rounded-lg font-medium hover:bg-red-500/30 transition-all duration-300">
                  Cancel
                </button>
              </div>
            </div>
          ))}
        </div>
      )}

      {activeTab === 'history' && (
        <div className="bg-slate-800/50 backdrop-blur-xl rounded-2xl border border-purple-500/20 p-6">
          <h3 className="text-xl font-semibold text-white mb-6">Payment History</h3>
          <div className="text-center py-12">
            <Clock className="w-12 h-12 text-slate-400 mx-auto mb-4" />
            <p className="text-slate-400">Payment history will be displayed here</p>
          </div>
        </div>
      )}

      {/* Auto-Pay Settings */}
      <div className="bg-gradient-to-r from-cyan-500/10 to-purple-500/10 backdrop-blur-xl rounded-2xl border border-cyan-500/20 p-6">
        <div className="flex items-center justify-between mb-4">
          <div>
            <h3 className="text-xl font-semibold text-white mb-2">Auto-Pay Settings</h3>
            <p className="text-slate-400">Automatically pay your bills on time</p>
          </div>
          <button className="bg-gradient-to-r from-cyan-500 to-purple-500 text-white px-6 py-2 rounded-xl font-medium hover:from-cyan-600 hover:to-purple-600 transition-all duration-300">
            Setup Auto-Pay
          </button>
        </div>
        
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <div className="text-center">
            <DollarSign className="w-8 h-8 text-cyan-400 mx-auto mb-2" />
            <p className="text-white font-semibold">5 Bills</p>
            <p className="text-slate-400 text-sm">Auto-pay enabled</p>
          </div>
          <div className="text-center">
            <CheckCircle className="w-8 h-8 text-green-400 mx-auto mb-2" />
            <p className="text-white font-semibold">100%</p>
            <p className="text-slate-400 text-sm">On-time payments</p>
          </div>
          <div className="text-center">
            <Calendar className="w-8 h-8 text-purple-400 mx-auto mb-2" />
            <p className="text-white font-semibold">$1,064.95</p>
            <p className="text-slate-400 text-sm">Next month total</p>
          </div>
        </div>
      </div>
    </div>
  );
};