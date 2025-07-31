import React, { useState } from 'react';
import { 
  Target, 
  TrendingUp, 
  PlusCircle, 
  Calendar,
  DollarSign,
  Home,
  Car,
  Utensils,
  ShoppingBag,
  Gamepad2,
  MoreHorizontal
} from 'lucide-react';

export const Budget: React.FC = () => {
  const [selectedPeriod, setSelectedPeriod] = useState('monthly');

  const budgetOverview = {
    totalBudget: 3500,
    spent: 2156.78,
    remaining: 1343.22,
    percentage: 62
  };

  const categoryBudgets = [
    { id: 1, category: 'Food & Dining', icon: Utensils, budgeted: 600, spent: 432.56, color: 'orange' },
    { id: 2, category: 'Transportation', icon: Car, budgeted: 400, spent: 298.43, color: 'blue' },
    { id: 3, category: 'Shopping', icon: ShoppingBag, budgeted: 500, spent: 389.12, color: 'pink' },
    { id: 4, category: 'Entertainment', icon: Gamepad2, budgeted: 300, spent: 156.78, color: 'purple' },
    { id: 5, category: 'Housing', icon: Home, budgeted: 1200, spent: 1200, color: 'green' },
  ];

  const savingsGoals = [
    { id: 1, name: 'Emergency Fund', target: 10000, current: 7500, color: 'green', icon: Target },
    { id: 2, name: 'Vacation Fund', target: 3000, current: 1250, color: 'blue', icon: Calendar },
    { id: 3, name: 'New Car', target: 25000, current: 12500, color: 'purple', icon: Car },
    { id: 4, name: 'House Down Payment', target: 50000, current: 18750, color: 'orange', icon: Home },
  ];

  const getColorClasses = (color: string) => {
    const colors: { [key: string]: { bg: string; text: string; border: string } } = {
      orange: { bg: 'bg-orange-500', text: 'text-orange-400', border: 'border-orange-500/30' },
      blue: { bg: 'bg-blue-500', text: 'text-blue-400', border: 'border-blue-500/30' },
      pink: { bg: 'bg-pink-500', text: 'text-pink-400', border: 'border-pink-500/30' },
      purple: { bg: 'bg-purple-500', text: 'text-purple-400', border: 'border-purple-500/30' },
      green: { bg: 'bg-green-500', text: 'text-green-400', border: 'border-green-500/30' },
    };
    return colors[color] || colors.purple;
  };

  return (
    <div className="p-8 space-y-8">
      {/* Header */}
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-bold text-white mb-2">Budget & Goals</h1>
          <p className="text-slate-400">Manage your spending and savings goals</p>
        </div>
        <div className="flex items-center gap-4">
          <select 
            value={selectedPeriod}
            onChange={(e) => setSelectedPeriod(e.target.value)}
            className="bg-slate-800/50 border border-purple-500/20 rounded-xl px-4 py-2 text-white focus:outline-none focus:border-purple-500/50"
          >
            <option value="weekly">Weekly</option>
            <option value="monthly">Monthly</option>
            <option value="yearly">Yearly</option>
          </select>
          <button className="bg-gradient-to-r from-purple-500 to-cyan-500 text-white px-6 py-2 rounded-xl font-medium hover:from-purple-600 hover:to-cyan-600 transition-all duration-300">
            Add Budget
          </button>
        </div>
      </div>

      {/* Budget Overview */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-1">
          <div className="bg-slate-800/50 backdrop-blur-xl rounded-2xl border border-purple-500/20 p-6">
            <h3 className="text-xl font-semibold text-white mb-6">Monthly Budget</h3>
            
            <div className="relative w-48 h-48 mx-auto mb-6">
              <svg className="w-48 h-48 transform -rotate-90" viewBox="0 0 100 100">
                <circle
                  cx="50"
                  cy="50"
                  r="40"
                  stroke="rgb(71 85 105)"
                  strokeWidth="8"
                  fill="none"
                />
                <circle
                  cx="50"
                  cy="50"
                  r="40"
                  stroke="url(#gradient)"
                  strokeWidth="8"
                  fill="none"
                  strokeLinecap="round"
                  strokeDasharray={`${budgetOverview.percentage * 2.51} 251`}
                  className="transition-all duration-1000 ease-out"
                />
                <defs>
                  <linearGradient id="gradient" x1="0%" y1="0%" x2="100%" y2="0%">
                    <stop offset="0%" stopColor="#8B5CF6" />
                    <stop offset="100%" stopColor="#06B6D4" />
                  </linearGradient>
                </defs>
              </svg>
              <div className="absolute inset-0 flex items-center justify-center flex-col">
                <span className="text-3xl font-bold text-white">{budgetOverview.percentage}%</span>
                <span className="text-slate-400 text-sm">Used</span>
              </div>
            </div>

            <div className="space-y-4">
              <div className="flex justify-between">
                <span className="text-slate-400">Total Budget</span>
                <span className="text-white font-semibold">${budgetOverview.totalBudget.toLocaleString()}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-slate-400">Spent</span>
                <span className="text-red-400 font-semibold">${budgetOverview.spent.toLocaleString()}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-slate-400">Remaining</span>
                <span className="text-green-400 font-semibold">${budgetOverview.remaining.toLocaleString()}</span>
              </div>
            </div>
          </div>
        </div>

        <div className="lg:col-span-2">
          <div className="bg-slate-800/50 backdrop-blur-xl rounded-2xl border border-purple-500/20 p-6">
            <div className="flex justify-between items-center mb-6">
              <h3 className="text-xl font-semibold text-white">Category Breakdown</h3>
              <button className="text-cyan-400 hover:text-cyan-300 text-sm font-medium">Manage Categories</button>
            </div>

            <div className="space-y-4">
              {categoryBudgets.map((category) => {
                const Icon = category.icon;
                const percentage = (category.spent / category.budgeted) * 100;
                const colors = getColorClasses(category.color);
                
                return (
                  <div key={category.id} className="space-y-3">
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-3">
                        <div className={`w-10 h-10 ${colors.bg}/20 rounded-xl flex items-center justify-center`}>
                          <Icon className={`w-5 h-5 ${colors.text}`} />
                        </div>
                        <span className="text-white font-medium">{category.category}</span>
                      </div>
                      <div className="text-right">
                        <p className="text-white font-semibold">${category.spent} / ${category.budgeted}</p>
                        <p className={`text-sm ${percentage > 80 ? 'text-red-400' : 'text-slate-400'}`}>
                          {percentage.toFixed(0)}% used
                        </p>
                      </div>
                    </div>
                    <div className="flex items-center gap-3">
                      <div className="flex-1 bg-slate-700 rounded-full h-2">
                        <div 
                          className={`${colors.bg} h-2 rounded-full transition-all duration-500 ${percentage > 100 ? 'bg-red-500' : ''}`}
                          style={{ width: `${Math.min(percentage, 100)}%` }}
                        ></div>
                      </div>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        </div>
      </div>

      {/* Savings Goals */}
      <div className="bg-slate-800/50 backdrop-blur-xl rounded-2xl border border-purple-500/20 p-6">
        <div className="flex justify-between items-center mb-6">
          <h3 className="text-xl font-semibold text-white">Savings Goals</h3>
          <button className="flex items-center gap-2 bg-gradient-to-r from-purple-500 to-cyan-500 text-white px-4 py-2 rounded-xl font-medium hover:from-purple-600 hover:to-cyan-600 transition-all duration-300">
            <PlusCircle className="w-4 h-4" />
            Add Goal
          </button>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          {savingsGoals.map((goal) => {
            const Icon = goal.icon;
            const percentage = (goal.current / goal.target) * 100;
            const colors = getColorClasses(goal.color);
            
            return (
              <div key={goal.id} className={`bg-slate-700/30 backdrop-blur-xl rounded-xl border ${colors.border} p-4 hover:bg-slate-700/50 transition-all duration-300`}>
                <div className="flex items-center justify-between mb-4">
                  <div className={`w-10 h-10 ${colors.bg}/20 rounded-xl flex items-center justify-center`}>
                    <Icon className={`w-5 h-5 ${colors.text}`} />
                  </div>
                  <button className="p-1 hover:bg-slate-600/50 rounded">
                    <MoreHorizontal className="w-4 h-4 text-slate-400" />
                  </button>
                </div>
                
                <h4 className="text-white font-semibold mb-2">{goal.name}</h4>
                
                <div className="space-y-2 mb-4">
                  <div className="flex justify-between text-sm">
                    <span className="text-slate-400">Progress</span>
                    <span className={colors.text}>{percentage.toFixed(0)}%</span>
                  </div>
                  <div className="w-full bg-slate-600 rounded-full h-2">
                    <div 
                      className={`${colors.bg} h-2 rounded-full transition-all duration-500`}
                      style={{ width: `${Math.min(percentage, 100)}%` }}
                    ></div>
                  </div>
                </div>
                
                <div className="flex justify-between text-sm">
                  <span className="text-slate-400">Current</span>
                  <span className="text-white font-medium">${goal.current.toLocaleString()}</span>
                </div>
                <div className="flex justify-between text-sm">
                  <span className="text-slate-400">Target</span>
                  <span className="text-white font-medium">${goal.target.toLocaleString()}</span>
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Spending Analytics */}
      <div className="bg-slate-800/50 backdrop-blur-xl rounded-2xl border border-purple-500/20 p-6">
        <div className="flex justify-between items-center mb-6">
          <h3 className="text-xl font-semibold text-white">Spending Trends</h3>
          <div className="flex items-center gap-2">
            <button className="px-3 py-1 bg-purple-500/20 text-purple-400 rounded-lg text-sm">6M</button>
            <button className="px-3 py-1 text-slate-400 hover:text-white rounded-lg text-sm">1Y</button>
            <button className="px-3 py-1 text-slate-400 hover:text-white rounded-lg text-sm">All</button>
          </div>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          <div className="lg:col-span-2">
            <div className="h-64 bg-slate-700/30 rounded-xl flex items-center justify-center">
              <div className="text-center">
                <TrendingUp className="w-12 h-12 text-slate-400 mx-auto mb-2" />
                <p className="text-slate-400">Spending trend chart would be displayed here</p>
              </div>
            </div>
          </div>
          
          <div className="space-y-4">
            <div className="bg-gradient-to-r from-green-500/10 to-emerald-500/10 rounded-xl border border-green-500/20 p-4">
              <div className="flex items-center gap-2 mb-2">
                <TrendingUp className="w-4 h-4 text-green-400" />
                <span className="text-green-400 font-medium text-sm">Best Month</span>
              </div>
              <p className="text-slate-300 text-sm mb-1">You saved $450 more than usual in December</p>
              <p className="text-green-400 font-semibold">-15% spending</p>
            </div>
            
            <div className="bg-gradient-to-r from-cyan-500/10 to-blue-500/10 rounded-xl border border-cyan-500/20 p-4">
              <div className="flex items-center gap-2 mb-2">
                <DollarSign className="w-4 h-4 text-cyan-400" />
                <span className="text-cyan-400 font-medium text-sm">Budget Insight</span>
              </div>
              <p className="text-slate-300 text-sm mb-1">You typically spend most on weekends</p>
              <p className="text-cyan-400 font-semibold">avg $89/weekend</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};