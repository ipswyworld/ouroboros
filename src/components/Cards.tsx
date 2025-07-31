import React, { useState } from 'react';
import { 
  CreditCard, 
  Plus, 
  MoreHorizontal, 
  Eye, 
  EyeOff,
  Lock,
  Unlock,
  Settings,
  TrendingUp,
  ShoppingBag,
  Utensils,
  Car,
  Gamepad2,
  Star,
  Shield,
  Smartphone
} from 'lucide-react';

export const Cards: React.FC = () => {
  const [selectedCard, setSelectedCard] = useState('1');
  const [showCardNumbers, setShowCardNumbers] = useState(false);

  const cards = [
    {
      id: '1',
      name: 'Primary Card',
      type: 'Visa',
      number: '4532 1234 5678 9012',
      expiryDate: '12/27',
      balance: 2456.78,
      limit: 5000,
      status: 'active',
      color: 'from-purple-600 to-blue-600',
      isPrimary: true
    },
    {
      id: '2',
      name: 'Business Card',
      type: 'Mastercard',
      number: '5432 9876 5432 1098',
      expiryDate: '09/26',
      balance: 1234.56,
      limit: 10000,
      status: 'active',
      color: 'from-cyan-600 to-teal-600',
      isPrimary: false
    },
    {
      id: '3',
      name: 'Savings Card',
      type: 'American Express',
      number: '3782 822463 10005',
      expiryDate: '03/28',
      balance: 567.89,
      limit: 3000,
      status: 'locked',
      color: 'from-orange-600 to-red-600',
      isPrimary: false
    }
  ];

  const cardTransactions = [
    { id: 1, merchant: 'Amazon', amount: -89.99, date: '2025-01-09', category: 'shopping', icon: ShoppingBag },
    { id: 2, merchant: 'Starbucks', amount: -5.67, date: '2025-01-09', category: 'food', icon: Utensils },
    { id: 3, merchant: 'Gas Station', amount: -45.00, date: '2025-01-08', category: 'transport', icon: Car },
    { id: 4, merchant: 'Netflix', amount: -15.99, date: '2025-01-08', category: 'entertainment', icon: Gamepad2 },
    { id: 5, merchant: 'Grocery Store', amount: -123.45, date: '2025-01-07', category: 'food', icon: Utensils },
  ];

  const cardInsights = [
    { label: 'Monthly Spending', value: '$1,246.78', change: '+12%', positive: false },
    { label: 'Cashback Earned', value: '$24.56', change: '+8%', positive: true },
    { label: 'Available Credit', value: '$3,543.22', change: '-5%', positive: true },
    { label: 'Payment Due', value: '$456.78', change: 'Jan 15', positive: false },
  ];

  const selectedCardData = cards.find(card => card.id === selectedCard);

  const getCategoryColor = (category: string) => {
    const colors: { [key: string]: string } = {
      shopping: 'text-pink-400',
      food: 'text-orange-400',
      transport: 'text-blue-400',
      entertainment: 'text-purple-400',
    };
    return colors[category] || 'text-slate-400';
  };

  const maskCardNumber = (cardNumber: string) => {
    if (!cardNumber || typeof cardNumber !== 'string') {
      return '•••• •••• •••• ••••';
    }
    return cardNumber.replace(/\d(?=\d{4})/g, '•');
  };

  const getCardGradient = (colorString: string) => {
    if (!colorString) return 'from-purple-600 to-blue-600';
    return colorString;
  };

  return (
    <div className="p-8 space-y-8">
      {/* Header */}
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-bold text-white mb-2">Cards & Wallet</h1>
          <p className="text-slate-400">Manage your payment methods and transactions</p>
        </div>
        <button className="bg-gradient-to-r from-purple-500 to-cyan-500 text-white px-6 py-2 rounded-xl font-medium hover:from-purple-600 hover:to-cyan-600 transition-all duration-300 flex items-center gap-2">
          <Plus className="w-4 h-4" />
          Add Card
        </button>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
        {/* Cards List */}
        <div className="lg:col-span-1">
          <div className="bg-slate-800/50 backdrop-blur-xl rounded-2xl border border-purple-500/20 p-6">
            <div className="flex justify-between items-center mb-6">
              <h3 className="text-xl font-semibold text-white">My Cards</h3>
              <button
                onClick={() => setShowCardNumbers(!showCardNumbers)}
                className="p-2 hover:bg-slate-700/50 rounded-lg transition-all duration-300"
              >
                {showCardNumbers ? <EyeOff className="w-5 h-5 text-slate-400" /> : <Eye className="w-5 h-5 text-slate-400" />}
              </button>
            </div>

            <div className="space-y-4">
              {cards.map((card) => (
                <div
                  key={card.id}
                  onClick={() => setSelectedCard(card.id)}
                  className={`relative cursor-pointer rounded-2xl p-6 bg-gradient-to-r ${getCardGradient(card.color)} transition-all duration-300 ${
                    selectedCard === card.id
                      ? 'ring-2 ring-purple-500/50 scale-105'
                      : 'hover:scale-102'
                  }`}
                >
                  <div className="flex justify-between items-start mb-8">
                    <div>
                      <p className="text-white/80 text-sm mb-1">{card.name}</p>
                      {card.isPrimary && (
                        <div className="flex items-center gap-1">
                          <Star className="w-3 h-3 text-yellow-400 fill-current" />
                          <span className="text-yellow-400 text-xs">Primary</span>
                        </div>
                      )}
                    </div>
                    <div className="flex items-center gap-2">
                      {card.status === 'locked' ? (
                        <Lock className="w-4 h-4 text-white/60" />
                      ) : (
                        <Shield className="w-4 h-4 text-white/60" />
                      )}
                      <button className="p-1 hover:bg-white/10 rounded">
                        <MoreHorizontal className="w-4 h-4 text-white/60" />
                      </button>
                    </div>
                  </div>

                  <div className="mb-6">
                    <p className="text-white text-lg font-mono tracking-wider">
                      {showCardNumbers ? card.number : maskCardNumber(card.number)}
                    </p>
                  </div>

                  <div className="flex justify-between items-end">
                    <div>
                      <p className="text-white/60 text-xs mb-1">VALID THRU</p>
                      <p className="text-white text-sm font-mono">{card.expiryDate}</p>
                    </div>
                    <div className="text-right">
                      <p className="text-white/60 text-xs mb-1">{card.type.toUpperCase()}</p>
                      <p className="text-white text-sm font-semibold">
                        ${card.balance.toLocaleString()}
                      </p>
                    </div>
                  </div>

                  {card.status === 'locked' && (
                    <div className="absolute inset-0 bg-black/40 backdrop-blur-sm rounded-2xl flex items-center justify-center">
                      <div className="text-center">
                        <Lock className="w-8 h-8 text-white mx-auto mb-2" />
                        <p className="text-white font-medium">Card Locked</p>
                      </div>
                    </div>
                  )}
                </div>
              ))}
            </div>

            {/* Digital Wallet Options */}
            <div className="mt-6 space-y-3">
              <h4 className="text-white font-semibold text-sm">Digital Wallets</h4>
              <div className="flex gap-3">
                <button className="flex-1 bg-slate-700/50 border border-slate-600/30 rounded-xl p-3 hover:bg-slate-600/50 transition-all duration-300">
                  <Smartphone className="w-5 h-5 text-cyan-400 mx-auto mb-1" />
                  <p className="text-white text-xs">Apple Pay</p>
                </button>
                <button className="flex-1 bg-slate-700/50 border border-slate-600/30 rounded-xl p-3 hover:bg-slate-600/50 transition-all duration-300">
                  <Smartphone className="w-5 h-5 text-green-400 mx-auto mb-1" />
                  <p className="text-white text-xs">Google Pay</p>
                </button>
              </div>
            </div>
          </div>
        </div>

        {/* Card Details and Transactions */}
        <div className="lg:col-span-2 space-y-6">
          {/* Card Insights */}
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            {cardInsights.map((insight, index) => (
              <div key={index} className="bg-slate-800/50 backdrop-blur-xl rounded-xl border border-purple-500/20 p-4">
                <p className="text-slate-400 text-sm mb-1">{insight.label}</p>
                <p className="text-white text-lg font-bold mb-1">{insight.value}</p>
                <div className="flex items-center gap-1">
                  {insight.change.includes('%') && (
                    <TrendingUp className={`w-3 h-3 ${insight.positive ? 'text-green-400' : 'text-red-400'}`} />
                  )}
                  <span className={`text-xs ${
                    insight.change.includes('%') 
                      ? insight.positive ? 'text-green-400' : 'text-red-400'
                      : 'text-slate-400'
                  }`}>
                    {insight.change}
                  </span>
                </div>
              </div>
            ))}
          </div>

          {/* Card Controls */}
          {selectedCardData && (
            <div className="bg-slate-800/50 backdrop-blur-xl rounded-2xl border border-purple-500/20 p-6">
              <div className="flex justify-between items-center mb-6">
                <h3 className="text-xl font-semibold text-white">Card Controls</h3>
                <span className={`px-3 py-1 rounded-lg text-sm font-medium ${
                  selectedCardData.status === 'active' 
                    ? 'bg-green-500/20 text-green-400' 
                    : 'bg-red-500/20 text-red-400'
                }`}>
                  {selectedCardData.status}
                </span>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                <button className={`flex items-center justify-center gap-2 p-4 rounded-xl border transition-all duration-300 ${
                  selectedCardData.status === 'active'
                    ? 'bg-red-500/10 border-red-500/30 text-red-400 hover:bg-red-500/20'
                    : 'bg-green-500/10 border-green-500/30 text-green-400 hover:bg-green-500/20'
                }`}>
                  {selectedCardData.status === 'active' ? <Lock className="w-4 h-4" /> : <Unlock className="w-4 h-4" />}
                  <span className="font-medium">
                    {selectedCardData.status === 'active' ? 'Lock Card' : 'Unlock Card'}
                  </span>
                </button>

                <button className="flex items-center justify-center gap-2 p-4 bg-slate-700/30 border border-slate-600/30 text-slate-300 rounded-xl hover:bg-slate-600/50 transition-all duration-300">
                  <Settings className="w-4 h-4" />
                  <span className="font-medium">Settings</span>
                </button>

                <button className="flex items-center justify-center gap-2 p-4 bg-slate-700/30 border border-slate-600/30 text-slate-300 rounded-xl hover:bg-slate-600/50 transition-all duration-300">
                  <TrendingUp className="w-4 h-4" />
                  <span className="font-medium">Limits</span>
                </button>

                <button className="flex items-center justify-center gap-2 p-4 bg-purple-500/10 border border-purple-500/30 text-purple-400 rounded-xl hover:bg-purple-500/20 transition-all duration-300">
                  <Star className="w-4 h-4" />
                  <span className="font-medium">Set Primary</span>
                </button>
              </div>

              {/* Credit Limit Progress */}
              <div className="mt-6">
                <div className="flex justify-between items-center mb-2">
                  <span className="text-slate-400 text-sm">Credit Utilization</span>
                  <span className="text-white text-sm font-medium">
                    ${selectedCardData.balance.toLocaleString()} / ${selectedCardData.limit.toLocaleString()}
                  </span>
                </div>
                <div className="w-full bg-slate-700 rounded-full h-2">
                  <div 
                    className="bg-gradient-to-r from-purple-500 to-cyan-500 h-2 rounded-full transition-all duration-500"
                    style={{ width: `${(selectedCardData.balance / selectedCardData.limit) * 100}%` }}
                  ></div>
                </div>
                <p className="text-slate-400 text-xs mt-1">
                  {((selectedCardData.balance / selectedCardData.limit) * 100).toFixed(1)}% utilized
                </p>
              </div>
            </div>
          )}

          {/* Recent Transactions */}
          <div className="bg-slate-800/50 backdrop-blur-xl rounded-2xl border border-purple-500/20 p-6">
            <div className="flex justify-between items-center mb-6">
              <h3 className="text-xl font-semibold text-white">Recent Transactions</h3>
              <button className="text-cyan-400 hover:text-cyan-300 text-sm font-medium">View All</button>
            </div>

            <div className="space-y-4">
              {cardTransactions.map((transaction) => {
                const Icon = transaction.icon;
                const colorClass = getCategoryColor(transaction.category);
                
                return (
                  <div key={transaction.id} className="flex items-center justify-between p-3 hover:bg-slate-700/30 rounded-xl transition-all duration-300">
                    <div className="flex items-center gap-3">
                      <div className={`w-10 h-10 bg-slate-700/50 rounded-xl flex items-center justify-center`}>
                        <Icon className={`w-5 h-5 ${colorClass}`} />
                      </div>
                      <div>
                        <p className="text-white font-medium">{transaction.merchant}</p>
                        <p className="text-slate-400 text-sm capitalize">{transaction.category}</p>
                      </div>
                    </div>
                    <div className="text-right">
                      <p className="text-white font-semibold">${Math.abs(transaction.amount).toFixed(2)}</p>
                      <p className="text-slate-400 text-sm">{transaction.date}</p>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};