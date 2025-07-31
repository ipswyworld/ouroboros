import React, { useState } from 'react';
import { Sidebar } from './components/Sidebar';
import { Dashboard } from './components/Dashboard';
import { Budget } from './components/Budget';
import { Bills } from './components/Bills';
import { Cards } from './components/Cards';
import { Profile } from './components/Profile';

function App() {
  const [activeSection, setActiveSection] = useState('dashboard');

  const renderActiveSection = () => {
    switch (activeSection) {
      case 'dashboard':
        return <Dashboard />;
      case 'budget':
        return <Budget />;
      case 'bills':
        return <Bills />;
      case 'cards':
        return <Cards />;
      case 'profile':
        return <Profile />;
      default:
        return <Dashboard />;
    }
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900">
      <div className="flex">
        <Sidebar activeSection={activeSection} setActiveSection={setActiveSection} />
        <main className="flex-1 min-h-screen">
          {renderActiveSection()}
        </main>
      </div>
    </div>
  );
}

export default App;