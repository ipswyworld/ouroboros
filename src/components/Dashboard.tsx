import React, { useState } from 'react';
import { 
  Eye, 
  EyeOff, 
  Send, 
  Plus, 
  Receipt, 
  MoreHorizontal,
  TrendingUp,
  TrendingDown,
  Bell,
  ArrowUpRight,
  ArrowDownLeft,
  User
} from 'lucide-react';

export const Dashboard: React.FC = () => {
  const [balanceVisible, setBalanceVisible] = useState(true);
  const balance = 12845.67;
  const monthlyChange = 12.5;

  const quickActions = [
    { label: 'Send Money', icon: Send, color: 'from-purple-500 to-purple-600' },
    { label: 'Request Money', icon: ArrowDownLeft, color: 'from-cyan-500 to-cyan-600' },
    { label: 'Add Money', icon: Plus, color: 'from-green-500 to-green-600' },
    { label: 'Pay Bills', icon: Receipt, color: 'from-orange-500 to-orange-600' },
  ];

  const recentTransactions = [
    { id: 1, type: 'expense', merchant: 'Netflix', amount: -15.99, date: '2025-01-09', category: 'Entertainment' },
    { id: 2, type: 'income', merchant: 'Salary Deposit', amount: 3500.00, date: '2025-01-08', category: 'Income' },
    { id: 3, type: 'expense', merchant: 'Grocery Store', amount: -89.43, date: '2025-01-08', category: 'Food' },
    { id: 4, type: 'expense', merchant: 'Gas Station', amount: -45.00, date: '2025-01-07', category: 'Transport' },
    { id: 5, type: 'income', merchant: 'Freelance Payment', amount: 250.00, date: '2025-01-06', category: 'Income' },
  ];

  const insights = [
    { category: 'Food & Dining', amount: 342.56, percentage: 28, color: 'bg-orange-500' },
    { category: 'Transportation', amount: 198.43, percentage: 16, color: 'bg-blue-500' },
    { category: 'Entertainment', amount: 156.78, percentage: 13, color: 'bg-purple-500' },
    { category: 'Shopping', amount: 289.12, percentage: 24, color: 'bg-pink-500' },
  ];

  return (
    <div className="p-8 space-y-8">
      {/* Header */}
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-bold text-white mb-2">Hello, Alex!</h1>
          <p className="text-slate-400">Thursday, January 9, 2025</p>
        </div>
        <div className="flex items-center gap-4">
          <button className="relative p-3 bg-slate-800/50 backdrop-blur-xl rounded-xl border border-purple-500/20 hover:bg-slate-700/50 transition-all duration-300">
            <Bell className="w-5 h-5 text-slate-400" />
            <span className="absolute top-1 right-1 w-2 h-2 bg-orange-500 rounded-full"></span>
          </button>
          <div className="w-10 h-10 bg-gradient-to-r from-purple-500 to-cyan-500 rounded-xl flex items-center justify-center">
            <User className="w-5 h-5 text-white" />
          </div>
        </div>
      </div>

      {/* Balance Card */}
      <div className="bg-gradient-to-r from-purple-600/20 to-cyan-600/20 backdrop-blur-xl rounded-2xl border border-purple-500/30 p-8">
        <div className="flex justify-between items-start mb-4">
          <div>
            <p className="text-slate-300 text-sm mb-1">Total Balance</p>
            <div className="flex items-center gap-4">
              <h2 className="text-4xl font-bold text-white">
                {balanceVisible ? `$${balance.toLocaleString('en-US', { minimumFractionDigits: 2 })}` : '••••••'}
              </h2>
              <button
                onClick={() => setBalanceVisible(!balanceVisible)}
                className="p-2 hover:bg-white/10 rounded-lg transition-all duration-300"
              >
                {balanceVisible ? <EyeOff className="w-5 h-5 text-slate-400" /> : <Eye className="w-5 h-5 text-slate-400" />}
              </button>
            </div>
          </div>
          <div className="flex items-center gap-2 text-green-400">
            <TrendingUp className="w-4 h-4" />
            <span className="text-sm font-medium">+{monthlyChange}% this month</span>
          </div>
        </div>
        
        <div className="flex justify-between text-sm">
          <div>
            <p className="text-slate-400">Income this month</p>
            <p className="text-green-400 font-semibold">+$4,250.00</p>
          </div>
          <div>
            <p className="text-slate-400">Expenses this month</p>
            <p className="text-red-400 font-semibold">-$1,892.34</p>
          </div>
        </div>
      </div>

      {/* Quick Actions */}
      <div className="grid grid-cols-4 gap-4">
        {quickActions.map((action, index) => {
          const Icon = action.icon;
          return (
            <button
              key={index}
              className="bg-slate-800/50 backdrop-blur-xl rounded-xl border border-purple-500/20 p-6 hover:bg-slate-700/50 transition-all duration-300 group"
            >
              <div className={`w-12 h-12 bg-gradient-to-r ${action.color} rounded-xl flex items-center justify-center mb-4 group-hover:scale-110 transition-transform duration-300`}>
                <Icon className="w-6 h-6 text-white" />
              </div>
              <p className="text-slate-300 font-medium">{action.label}</p>
            </button>
          );
        })}
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
        {/* Recent Transactions */}
        <div className="bg-slate-800/50 backdrop-blur-xl rounded-2xl border border-purple-500/20 p-6">
          <div className="flex justify-between items-center mb-6">
            <h3 className="text-xl font-semibold text-white">Recent Activity</h3>
            <button className="text-cyan-400 hover:text-cyan-300 text-sm font-medium">See All</button>
          </div>
          
          <div className="space-y-4">
            {recentTransactions.map((transaction) => (
              <div key={transaction.id} className="flex items-center justify-between p-3 hover:bg-slate-700/30 rounded-xl transition-all duration-300">
                <div className="flex items-center gap-3">
                  <div className={`w-10 h-10 rounded-xl flex items-center justify-center ${
                    transaction.type === 'income' ? 'bg-green-500/20 text-green-400' : 'bg-red-500/20 text-red-400'
                  }`}>
                    {transaction.type === 'income' ? <ArrowDownLeft className="w-5 h-5" /> : <ArrowUpRight className="w-5 h-5" />}
                  </div>
                  <div>
                    <p className="text-white font-medium">{transaction.merchant}</p>
                    <p className="text-slate-400 text-sm">{transaction.category}</p>
                  </div>
                </div>
                <div className="text-right">
                  <p className={`font-semibold ${transaction.type === 'income' ? 'text-green-400' : 'text-white'}`}>
                    {transaction.type === 'income' ? '+' : ''}${Math.abs(transaction.amount).toFixed(2)}
                  </p>
                  <p className="text-slate-400 text-sm">{transaction.date}</p>
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Financial Insights */}
        <div className="bg-slate-800/50 backdrop-blur-xl rounded-2xl border border-purple-500/20 p-6">
          <div className="flex justify-between items-center mb-6">
            <h3 className="text-xl font-semibold text-white">Spending Insights</h3>
            <button className="p-2 hover:bg-slate-700/50 rounded-lg">
              <MoreHorizontal className="w-5 h-5 text-slate-400" />
            </button>
          </div>
          
          <div className="space-y-4">
            {insights.map((insight, index) => (
              <div key={index} className="space-y-2">
                <div className="flex justify-between items-center">
                  <span className="text-slate-300 font-medium">{insight.category}</span>
                  <span className="text-white font-semibold">${insight.amount}</span>
                </div>
                <div className="flex items-center gap-3">
                  <div className="flex-1 bg-slate-700 rounded-full h-2">
                    <div 
                      className={`${insight.color} h-2 rounded-full transition-all duration-500`}
                      style={{ width: `${insight.percentage}%` }}
                    ></div>
                  </div>
                  <span className="text-slate-400 text-sm">{insight.percentage}%</span>
                </div>
              </div>
            ))}
          </div>

          <div className="mt-6 p-4 bg-gradient-to-r from-cyan-500/10 to-purple-500/10 rounded-xl border border-cyan-500/20">
            <div className="flex items-center gap-2 mb-2">
              <TrendingDown className="w-4 h-4 text-cyan-400" />
              <span className="text-cyan-400 font-medium text-sm">Money Saving Tip</span>
            </div>
            <p className="text-slate-300 text-sm">You spent 15% less on dining this month. Keep it up!</p>
          </div>
        </div>
      </div>

      {/* Promotional Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div className="bg-gradient-to-r from-orange-500/20 to-red-500/20 backdrop-blur-xl rounded-2xl border border-orange-500/30 p-6">
          <h4 className="text-lg font-semibold text-white mb-2">Cashback Rewards</h4>
          <p className="text-slate-300 text-sm mb-4">Earn 3% cashback on all dining purchases this month</p>
          <button className="bg-orange-500 hover:bg-orange-600 text-white px-4 py-2 rounded-lg font-medium transition-all duration-300">
            Learn More
          </button>
        </div>
        
        <div className="bg-gradient-to-r from-green-500/20 to-emerald-500/20 backdrop-blur-xl rounded-2xl border border-green-500/30 p-6">
          <h4 className="text-lg font-semibold text-white mb-2">Savings Challenge</h4>
          <p className="text-slate-300 text-sm mb-4">Join the $1000 savings challenge and win rewards</p>
          <button className="bg-green-500 hover:bg-green-600 text-white px-4 py-2 rounded-lg font-medium transition-all duration-300">
            Join Now
          </button>
        </div>
      </div>
    </div>
  );
};