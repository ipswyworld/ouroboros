import { invoke } from '@tauri-apps/api/tauri';

// Global state
let wallet = null;
let nodeUrl = localStorage.getItem('nodeUrl') || 'http://localhost:8001';

// Initialize app
window.addEventListener('DOMContentLoaded', async () => {
    // Set node URL
    document.getElementById('node-url').value = nodeUrl;

    // Try to load existing wallet
    try {
        wallet = await invoke('get_wallet_info');
        showDashboard();
        loadDashboardData();
    } catch (error) {
        showWelcomeScreen();
    }

    // Setup navigation
    setupNavigation();
});

// Navigation
function setupNavigation() {
    document.querySelectorAll('.nav-item').forEach((item) => {
        item.addEventListener('click', () => {
            const page = item.dataset.page;
            window.switchPage(page);
        });
    });
}

window.switchPage = function (pageName) {
    // Hide all pages
    document.querySelectorAll('.page').forEach((page) => {
        page.style.display = 'none';
    });

    // Show selected page
    document.getElementById(pageName).style.display = 'block';

    // Update active nav item
    document.querySelectorAll('.nav-item').forEach((item) => {
        item.classList.remove('active');
    });
    document.querySelector(`[data-page="${pageName}"]`)?.classList.add('active');

    // Load page data
    if (pageName === 'microchains') {
        refreshMicrochains();
    } else if (pageName === 'history') {
        refreshHistory();
    } else if (pageName === 'send') {
        loadSendPage();
    } else if (pageName === 'receive') {
        document.getElementById('receive-address').value = wallet?.address || '';
    }
};

// Wallet Management
window.showCreateWallet = function () {
    const modalBody = document.getElementById('modal-body');
    modalBody.innerHTML = `
        <h2 class="mb-16">Create New Wallet</h2>
        <form onsubmit="createWallet(event)">
            <div class="form-group">
                <label>Wallet Name (optional)</label>
                <input type="text" id="wallet-name" placeholder="My Wallet">
            </div>
            <button type="submit" class="btn btn-primary">Create Wallet</button>
        </form>
    `;
    showModal();
};

window.showImportWallet = function () {
    const modalBody = document.getElementById('modal-body');
    modalBody.innerHTML = `
        <h2 class="mb-16">Import Wallet</h2>
        <div class="form-group mb-16">
            <button class="btn btn-secondary" onclick="showImportFromMnemonic()">
                Import from Recovery Phrase
            </button>
        </div>
        <div class="form-group">
            <button class="btn btn-secondary" onclick="showImportFromKey()">
                Import from Private Key
            </button>
        </div>
    `;
    showModal();
};

window.showImportFromMnemonic = function () {
    const modalBody = document.getElementById('modal-body');
    modalBody.innerHTML = `
        <h2 class="mb-16">Import from Recovery Phrase</h2>
        <form onsubmit="importFromMnemonic(event)">
            <div class="form-group">
                <label>12-word Recovery Phrase</label>
                <textarea id="import-mnemonic" rows="3" required></textarea>
            </div>
            <div class="form-group">
                <label>Wallet Name (optional)</label>
                <input type="text" id="import-name">
            </div>
            <button type="submit" class="btn btn-primary">Import Wallet</button>
        </form>
    `;
    showModal();
};

window.showImportFromKey = function () {
    const modalBody = document.getElementById('modal-body');
    modalBody.innerHTML = `
        <h2 class="mb-16">Import from Private Key</h2>
        <form onsubmit="importFromKey(event)">
            <div class="form-group">
                <label>Private Key (hex)</label>
                <textarea id="import-key" rows="2" required></textarea>
            </div>
            <div class="form-group">
                <label>Wallet Name (optional)</label>
                <input type="text" id="import-key-name">
            </div>
            <button type="submit" class="btn btn-primary">Import Wallet</button>
        </form>
    `;
    showModal();
};

window.createWallet = async function (event) {
    event.preventDefault();
    const name = document.getElementById('wallet-name').value || null;

    try {
        const result = await invoke('create_wallet', { name });
        wallet = result.wallet;

        // Show mnemonic
        const modalBody = document.getElementById('modal-body');
        modalBody.innerHTML = `
            <h2 class="mb-16">⚠️ Save Your Recovery Phrase</h2>
            <p class="mb-16" style="color: var(--text-secondary);">
                Write down these 12 words in order. This is the ONLY way to recover your wallet.
            </p>
            <div style="background: var(--bg-card); padding: 16px; border-radius: 8px; margin-bottom: 24px;">
                <code style="font-size: 14px;">${result.mnemonic}</code>
            </div>
            <div class="form-group">
                <label>
                    <input type="checkbox" id="confirm-saved">
                    I have securely saved my recovery phrase
                </label>
            </div>
            <button class="btn btn-primary" onclick="finishWalletSetup()">Continue</button>
        `;
    } catch (error) {
        alert('Error creating wallet: ' + error);
    }
};

window.importFromMnemonic = async function (event) {
    event.preventDefault();
    const mnemonic = document.getElementById('import-mnemonic').value.trim();
    const name = document.getElementById('import-name').value || null;

    try {
        wallet = await invoke('import_wallet', { mnemonic, name });
        finishWalletSetup();
    } catch (error) {
        alert('Error importing wallet: ' + error);
    }
};

window.importFromKey = async function (event) {
    event.preventDefault();
    const privateKey = document.getElementById('import-key').value.trim();
    const name = document.getElementById('import-key-name').value || null;

    try {
        wallet = await invoke('import_from_key', { privateKey, name });
        finishWalletSetup();
    } catch (error) {
        alert('Error importing wallet: ' + error);
    }
};

window.finishWalletSetup = function () {
    closeModal();
    showDashboard();
    loadDashboardData();
};

// Dashboard
function showWelcomeScreen() {
    document.getElementById('welcome-screen').style.display = 'block';
}

function showDashboard() {
    document.getElementById('welcome-screen').style.display = 'none';
    document.getElementById('dashboard').style.display = 'block';
}

async function loadDashboardData() {
    // Load wallet info
    document.getElementById('wallet-address').textContent = wallet.address;
    document.getElementById('wallet-pubkey').textContent = wallet.public_key.substring(0, 32) + '...';
    document.getElementById('wallet-name').textContent = wallet.name || 'Unnamed Wallet';

    // Load balance
    await refreshBalance();
}

window.refreshBalance = async function () {
    try {
        const balance = await invoke('get_balance', { nodeUrl });
        document.getElementById('main-balance').textContent = (balance.balance / 100).toFixed(2);
        document.getElementById('pending-balance').textContent =
            `Pending: ${(balance.pending / 100).toFixed(2)} OURO`;
    } catch (error) {
        console.error('Error fetching balance:', error);
        document.getElementById('main-balance').textContent = 'Error';
    }
};

// Send Page
function loadSendPage() {
    updateSendForm();
}

window.updateSendForm = function () {
    const chain = document.getElementById('send-chain').value;
    const microchainSelect = document.getElementById('microchain-select');

    if (chain === 'microchain') {
        microchainSelect.style.display = 'block';
        loadMicrochainsForSend();
    } else {
        microchainSelect.style.display = 'none';
    }
};

async function loadMicrochainsForSend() {
    try {
        const microchains = await invoke('list_microchains', { nodeUrl });
        const select = document.getElementById('send-microchain-id');
        select.innerHTML = microchains
            .map((mc) => `<option value="${mc.id}">${mc.name} (${mc.id})</option>`)
            .join('');
    } catch (error) {
        console.error('Error loading microchains:', error);
    }
}

window.sendTransaction = async function (event) {
    event.preventDefault();

    const chain = document.getElementById('send-chain').value;
    const to = document.getElementById('send-to').value;
    const amount = Math.floor(parseFloat(document.getElementById('send-amount').value) * 100);

    try {
        let txId;
        if (chain === 'mainchain') {
            txId = await invoke('send_transaction', { nodeUrl, to, amount });
        } else {
            const microchainId = document.getElementById('send-microchain-id').value;
            txId = await invoke('send_microchain_transaction', {
                nodeUrl,
                microchainId,
                to,
                amount,
            });
        }

        alert(`Transaction sent! TX ID: ${txId}`);
        event.target.reset();
        refreshBalance();
    } catch (error) {
        alert('Transaction failed: ' + error);
    }
};

// Microchains
window.refreshMicrochains = async function () {
    try {
        const microchains = await invoke('list_microchains', { nodeUrl });
        const list = document.getElementById('microchains-list');

        if (microchains.length === 0) {
            list.innerHTML = '<div class="empty-state">No microchains found</div>';
            return;
        }

        list.innerHTML = microchains
            .map(
                (mc) => `
            <div class="microchain-card">
                <h3>${mc.name}</h3>
                <div class="microchain-info">
                    <div><strong>ID:</strong> ${mc.id.substring(0, 16)}...</div>
                    <div><strong>Owner:</strong> ${mc.owner.substring(0, 16)}...</div>
                    <div><strong>Height:</strong> ${mc.block_height}</div>
                    <div><button class="btn btn-small" onclick="viewMicrochainBalance('${mc.id}')">
                        View Balance
                    </button></div>
                </div>
            </div>
        `
            )
            .join('');
    } catch (error) {
        console.error('Error loading microchains:', error);
    }
};

window.viewMicrochainBalance = async function (microchainId) {
    try {
        const balance = await invoke('get_microchain_balance', { nodeUrl, microchainId });
        alert(`Balance: ${(balance / 100).toFixed(2)} tokens`);
    } catch (error) {
        alert('Error fetching balance: ' + error);
    }
};

// History
window.refreshHistory = async function () {
    try {
        const transactions = await invoke('get_transaction_history', { nodeUrl });
        const list = document.getElementById('history-list');

        if (transactions.length === 0) {
            list.innerHTML = '<div class="empty-state">No transactions yet</div>';
            return;
        }

        list.innerHTML = transactions
            .map(
                (tx) => `
            <div class="history-item">
                <div class="history-item-left">
                    <div class="history-item-title">
                        ${tx.from === wallet.address ? 'Sent' : 'Received'}
                    </div>
                    <div class="history-item-subtitle">
                        ${tx.from === wallet.address ? 'To: ' + tx.to.substring(0, 16) : 'From: ' + tx.from.substring(0, 16)}...
                    </div>
                </div>
                <div class="history-item-right">
                    <div class="history-item-amount">
                        ${tx.from === wallet.address ? '-' : '+'}${(tx.amount / 100).toFixed(2)} OURO
                    </div>
                    <div class="history-item-status status-${tx.status}">
                        ${tx.status}
                    </div>
                </div>
            </div>
        `
            )
            .join('');
    } catch (error) {
        console.error('Error loading history:', error);
    }
};

// Node Linking
window.linkToNode = async function () {
    try {
        const result = await invoke('link_to_node', { nodeUrl });
        alert(result);
    } catch (error) {
        alert('Linking failed: ' + error);
    }
};

// Settings
window.saveNodeUrl = function () {
    const url = document.getElementById('node-url').value;
    nodeUrl = url;
    localStorage.setItem('nodeUrl', url);
    alert('Node URL saved!');
};

window.exportMnemonic = async function () {
    if (!confirm('Are you sure you want to export your recovery phrase?')) {
        return;
    }

    try {
        const mnemonic = await invoke('export_mnemonic');
        const modalBody = document.getElementById('modal-body');
        modalBody.innerHTML = `
            <h2 class="mb-16">Recovery Phrase</h2>
            <p class="mb-16" style="color: var(--text-secondary);">
                Keep this safe! Anyone with this phrase can access your wallet.
            </p>
            <div style="background: var(--bg-card); padding: 16px; border-radius: 8px; margin-bottom: 24px;">
                <code style="font-size: 14px;">${mnemonic}</code>
            </div>
            <button class="btn btn-primary" onclick="closeModal()">Close</button>
        `;
        showModal();
    } catch (error) {
        alert('Error exporting mnemonic: ' + error);
    }
};

window.confirmDeleteWallet = function () {
    if (confirm('Are you sure you want to delete your wallet? This cannot be undone!')) {
        // In production, call backend to delete wallet file
        alert('Wallet deletion not yet implemented');
    }
};

// Utilities
window.copyAddress = function () {
    const address = wallet?.address || '';
    navigator.clipboard.writeText(address);
    alert('Address copied to clipboard!');
};

function showModal() {
    document.getElementById('modal').style.display = 'flex';
}

window.closeModal = function () {
    document.getElementById('modal').style.display = 'none';
};
