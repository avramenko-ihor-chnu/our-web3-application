// Відображення активних хеджів у Dashboard

let userHedges = JSON.parse(localStorage.getItem('userHedges') || '[]');

function saveHedges() {
    localStorage.setItem('userHedges', JSON.stringify(userHedges));
}

function renderHedges() {
    const tbody = document.getElementById('hedgeTableBody');
    if (!tbody) return;

    if (userHedges.length === 0) {
        tbody.innerHTML = '<tr><td colspan="6">No active hedges yet.</td></tr>';
        return;
    }

    tbody.innerHTML = userHedges.map((h, i) => `
        <tr>
            <td>${h.market}</td>
            <td>${h.size}</td>
            <td>${h.toWin}</td>
            <td>${h.toPay}</td>
            <td>${h.status}</td>
            <td>
                ${h.status === 'Active' ? 
                    `<button class="btn btn-gradient btn-round" data-index="${i}" onclick="closeHedge(${i})">Close</button>` 
                    : '-'}
            </td>
        </tr>
    `).join('');
}

function closeHedge(index) {
    const hedge = userHedges[index];
    if (!confirm(`Close hedge: ${hedge.market}?`)) return;

    hedge.status = 'Closed';
    hedge.closePrice = (Math.random() * 100 + 10).toFixed(2); // умовна ціна закриття
    hedge.closedAt = new Date().toLocaleString();

    saveHedges();
    renderHedges();

    alert(`Hedge closed at $${hedge.closePrice}`);
}

//=================================================================================================

// <!-- JS Popup Logic -->
const soundsuccess = document.getElementById('popup-sound');
const sounderror = document.getElementById('popup-error-sound');

function openPopup(id) {
    document.querySelectorAll('.popup-overlay').forEach(o => o.style.display = 'none');
    const el = document.getElementById(id);
    if (el) el.style.display = 'flex';

    if (id === 'popup-hedge-result-overlay') {
        soundsuccess?.play().catch(() => { });
    }

    if (id === 'popup-hedge-error-overlay') {
        sounderror?.play().catch(() => { });
    }
}

function closePopup(id) {
    const el = document.getElementById(id);
    if (el) el.style.display = 'none';
}

document.addEventListener('click', e => {
    if (e.target.classList.contains('popup-overlay')) {
        e.target.style.display = 'none';
    }
});

//=================================================================================================

// <!-- JS Hedge Buttons -->
document.addEventListener('DOMContentLoaded', () => {
    let hedgeTimeout = null;
    let isProcessing = false;

    const addBtn = document.getElementById('addHedgeBtn');
    if (addBtn) {
        addBtn.addEventListener('click', e => {
            e.preventDefault();
            if (isProcessing) return;
            isProcessing = true;

            document.querySelectorAll('.popup-overlay').forEach(o => o.style.display = 'none');
            openPopup('popup-overlay'); // Loading popup

            hedgeTimeout = setTimeout(() => {
                closePopup('popup-overlay');
                openPopup('popup-hedge-offer-overlay'); // Offer popup
                isProcessing = false;
            }, 2000);
        });
    }

    const hedgeNowBtn = document.getElementById('hedgeNowBtn');
    if (hedgeNowBtn) {
        hedgeNowBtn.addEventListener('click', e => {
            e.preventDefault();
            closePopup('popup-hedge-offer-overlay');
            openPopup('popup-hedge-confirmation-overlay');
            // Імітація "обробки" транзакції
            setTimeout(() => {
                closePopup('popup-hedge-confirmation-overlay');
                // Рандомний результат: успіх чи помилка
                const isSuccess = Math.random() < 0.6; // 60% шанс успіху
                if (isSuccess) {
                    // Імітуємо дані з popup-hedge-offer
                    const selectedHedge = document.querySelector('input[name="hedge-select"]:checked');
                    const row = selectedHedge?.closest('tr');
                    if (row) {
                        const market = row.cells[0].innerText;
                        const size = row.cells[1].innerText;
                        const toWin = row.cells[2].innerText;
                        const toPay = row.cells[4].innerText;
                        const hedge = {
                            market, size, toWin, toPay,
                            status: 'Active',
                            date: new Date().toLocaleString()
                        };
                        userHedges.push(hedge);
                        saveHedges();
                        renderHedges();
                    }

                    openPopup('popup-hedge-result-overlay');
                } else {
                    openPopup('popup-hedge-error-overlay');
                }

            }, 2000 + Math.random() * 2000); // випадкова затримка 2-4 сек.
        });
    }

renderHedges(); // Ініціалізація при завантаженні

});

//=================================================================================================

// <!-- JS Connecting wallets -->
document.addEventListener('DOMContentLoaded', () => {
    const connectBtns = [
        document.getElementById('connectWalletBtn'),
        document.getElementById('connectBtn')
    ];
    const statusEl = document.getElementById('walletStatus');
    const walletCapEl = document.getElementById('wallet-cap');
    const dashboardEl = document.getElementById('Dashboard');
    const referralEl = document.getElementById('Referral');
    const phantomBtn = document.getElementById('wallet-phantom');
    const backpackBtn = document.getElementById('wallet-backpack');
    const manualBtn = document.getElementById('manualWalletBtn');
    const connectBlock = document.getElementById('ConnectWallet');

    // --- Відкриття попапа при кліку на будь-яку кнопку ---
    connectBtns.forEach(btn => {
        if (!btn) return;
        btn.addEventListener('click', () => openPopup('popup-wallet-overlay'));
    });

    // --- Універсальна функція після підключення ---
    async function handleWalletConnected(fullAddress) {
        const short = fullAddress.slice(0, 4) + '...' + fullAddress.slice(-4);
        connectBtns.forEach(b => b.textContent = short);
        statusEl.textContent = 'Connected: ' + fullAddress;

        // --- Показуємо основний контент ---
        if (connectBlock) connectBlock.hidden = true;
        walletCapEl.hidden = false;
        dashboardEl.hidden = false;
        referralEl.hidden = false;

        // --- Завантажуємо позиції користувача ---
        if (typeof loadUserPositions === 'function') {
            try {
                await loadUserPositions(fullAddress);
            } catch (e) {
                console.error('Error loading positions:', e);
            }
        }
    }

    // --- Phantom Wallet ---
    phantomBtn?.addEventListener('click', async () => {
        closePopup('popup-wallet-overlay');
        try {
            if (!window.solana?.isPhantom) {
                alert('Please install Phantom Wallet');
                return;
            }
            const resp = await window.solana.connect();
            await handleWalletConnected(resp.publicKey.toString());
        } catch (err) { console.error(err); }
    });

    // --- Backpack / MetaMask ---
    backpackBtn?.addEventListener('click', async () => {
        closePopup('popup-wallet-overlay');
        try {
            if (!window.ethereum) {
                alert('Please install Backpack or MetaMask');
                return;
            }
            const accounts = await window.ethereum.request({ method: 'eth_requestAccounts' });
            await handleWalletConnected(accounts[0]);
        } catch (err) { console.error(err); }
    });

    // --- Manual Wallet Input ---
    manualBtn?.addEventListener('click', () => {
        const input = document.getElementById('manualWalletInput').value.trim();
        if (input.length < 10) {
            alert('Please enter a valid wallet address');
            return;
        }
        closePopup('popup-wallet-overlay');
        handleWalletConnected(input); // викликаємо універсальну функцію
    });
});

//=================================================================================================

// <!-- JS Crypto Chart -->
const currencies = {
    SOL: "solana",
    USDC: "usd-coin",
    MET: "meteora"
};
const pointsPerDay = 4;
const forecastDays = 3;
const trainDays = 7;

async function loadData(coinId) {
    const res = await fetch(`https://api.coingecko.com/api/v3/coins/${coinId}/market_chart?vs_currency=usd&days=${trainDays}`);
    const data = await res.json();
    return data.prices.map(p => ({ time: new Date(p[0]), price: p[1] }));
}

function localRegression(prices, windowSize, predictCount) {
    const result = [...prices.map(p => p.price)];
    for (let i = 0; i < predictCount; i++) {
        const start = Math.max(result.length - windowSize, 0);
        const ys = result.slice(start);
        const xs = Array.from({ length: ys.length }, (_, j) => j);
        const n = xs.length;
        const sumX = xs.reduce((a, b) => a + b, 0);
        const sumY = ys.reduce((a, b) => a + b, 0);
        const sumXY = xs.reduce((a, b, j) => a + b * ys[j], 0);
        const sumX2 = xs.reduce((a, b) => a + b * b, 0);
        const slope = (n * sumXY - sumX * sumY) / (n * sumX2 - sumX * sumX);
        const intercept = (sumY - slope * sumX) / n;
        result.push(slope * ys.length + intercept);
    }
    return result.slice(prices.length);
}

async function buildChart(selectedCurrency) {
    const prices = await loadData(currencies[selectedCurrency]);
    const windowSize = 6;
    const predictPoints = forecastDays * pointsPerDay;
    const msStep = 6 * 60 * 60 * 1000;

    const predicted = localRegression(prices, windowSize, predictPoints);
    const forecastData = Array(prices.length - 1).fill(null);
    forecastData.push(prices[prices.length - 1].price);
    forecastData.push(...predicted);

    const emaPeriod = 5;
    const alpha = 2 / (emaPeriod + 1);
    const emaValues = [];
    for (let i = 0; i < prices.length; i++) {
        if (i === 0) emaValues.push(prices[i].price);
        else emaValues.push(alpha * prices[i].price + (1 - alpha) * emaValues[i - 1]);
    }
    let lastEMA = emaValues[emaValues.length - 1];
    for (let i = 0; i < predicted.length; i++) {
        lastEMA = alpha * predicted[i] + (1 - alpha) * lastEMA;
        emaValues.push(lastEMA);
    }

    const labels = prices.map(p => p.time.toLocaleString('en-GB', {
        hour12: false, hour: '2-digit', minute: '2-digit', day: '2-digit', month: '2-digit'
    })).concat(predicted.map((_, i) =>
        new Date(prices[prices.length - 1].time.getTime() + (i + 1) * msStep)
            .toLocaleString('en-GB', { hour12: false, hour: '2-digit', minute: '2-digit', day: '2-digit', month: '2-digit' })
    ));

    const buyPrice = prices[0].price;
    const sellPrice = prices[prices.length - 1].price;
    const breakEven = buyPrice / 1.2;

    const backgroundPlugin = {
        id: 'background_zones',
        beforeDraw: (chart) => {
            const ctx = chart.ctx;
            const yScale = chart.scales.y;
            const area = chart.chartArea;
            const yBuy = yScale.getPixelForValue(buyPrice);
            const ySell = yScale.getPixelForValue(sellPrice);
            const yBreak = yScale.getPixelForValue(breakEven);
            ctx.save();
            ctx.fillStyle = 'rgba(16,185,129,0.12)';
            ctx.fillRect(area.left, yBuy, area.right - area.left, area.bottom - yBuy);
            ctx.fillStyle = 'rgba(239,68,68,0.12)';
            ctx.fillRect(area.left, area.top, area.right - area.left, ySell - area.top);
            ctx.fillStyle = 'rgba(250,204,21,0.15)';
            ctx.fillRect(area.left, Math.min(yBuy, yBreak), area.right - area.left, Math.abs(yBreak - yBuy));
            ctx.restore();
        }
    };

    const ctx = document.getElementById('cryptoChart').getContext('2d');
    if (window.chart) window.chart.destroy();
    window.chart = new Chart(ctx, {
        type: 'line',
        data: {
            labels,
            datasets: [
                { label: selectedCurrency + ' Price', data: prices.map(p => p.price), borderColor: '#38bdf8', backgroundColor: 'rgba(56,189,248,0.25)', tension: 0.3, pointRadius: 0 },
                { label: 'Forecast', data: forecastData, borderColor: '#a78bfa', borderDash: [6, 4], borderWidth: 2, tension: 0.4, pointRadius: 2, pointBackgroundColor: '#a78bfa', hidden: !document.getElementById('toggleForecast').checked },
                { label: 'EMA', data: emaValues, borderColor: '#f472b6', borderWidth: 2, pointRadius: 0, tension: 0.4, hidden: !document.getElementById('toggleEMA').checked },
                { label: 'Buy Price', data: prices.concat(predicted).map(() => buyPrice), borderColor: '#10b981', borderDash: [5, 5], borderWidth: 1, pointRadius: 0 },
                { label: 'Sell Price', data: prices.concat(predicted).map(() => sellPrice), borderColor: '#ef4444', borderDash: [5, 5], borderWidth: 1, pointRadius: 0 },
                { label: 'Break-even', data: prices.concat(predicted).map(() => breakEven), borderColor: '#facc15', borderDash: [5, 5], borderWidth: 1, pointRadius: 0 }
            ]
        },
        options: {
            responsive: true,
            plugins: {
                legend: { labels: { color: '#e2e8f0' } },
                title: { display: true, text: selectedCurrency + ' — Forecast', color: '#e2e8f0', font: { size: 16 } }
            },
            scales: { x: { ticks: { color: '#94a3b8' } }, y: { ticks: { color: '#94a3b8' } } }
        },
        plugins: [backgroundPlugin]
    });
}

document.getElementById('toggleForecast').addEventListener('change', e => {
    if (window.chart) window.chart.data.datasets[1].hidden = !e.target.checked;
    if (window.chart) window.chart.update();
});
document.getElementById('toggleEMA').addEventListener('change', e => {
    if (window.chart) window.chart.data.datasets[2].hidden = !e.target.checked;
    if (window.chart) window.chart.update();
});
document.querySelectorAll('input[name="currency"]').forEach(radio => {
    radio.addEventListener('change', e => buildChart(e.target.value));
});
buildChart('SOL');

//=================================================================================================

async function loadUserPositions(walletAddress) {
    const tbody = document.querySelector("#MyPositions .hedge-table tbody");
    tbody.innerHTML = "";
    try {
        const params = new URLSearchParams();
        params.append("account_id", walletAddress);

        const resp = await fetch("/positions", {
            method: "POST",
            headers: {
                "Content-Type": "application/x-www-form-urlencoded"
            },
            body: params.toString()
        });
        const html = await resp.text();
        tbody.innerHTML = html; // вставка таблиці
        // --- Підтягуємо іконки після вставки ---
        tbody.querySelectorAll(".token-icon").forEach(img => {
            const symbol = img.dataset.symbol;
            img.src = getTokenIcon(symbol);
            img.width = 24;
            img.height = 24;
            img.style.marginRight = "8px";
            img.style.verticalAlign = "middle";
        });
    } catch (err) {
        console.error("Ошибка при загрузке позиций:", err);
    }
}

//=================================================================================================

// <!-- BACKGROUND PARTICLES -->
particlesJS("particles-js", {
    particles: {
        number: { value: 69 },
        color: { value: "#ec6320" },
        shape: { type: "circle" },
        opacity: { value: 0.2 },
        size: { value: 2 },
        line_linked: {
            enable: true,
            distance: 200,
            color: "#ec6320",
            opacity: 0.2,
            width: 1
        },
        move: {
            enable: true,
            speed: 1 
        }
    },
    interactivity: {
        detect_on: "canvas",
        events: {
            onhover: { enable: true, mode: "repulse" },
            onclick: { enable: true, mode: "push" },
            resize: true
        },
        modes: {
            repulse: { distance: 80, duration: 0.4 },
            push: { particles_nb: 2 }
        }
    },
    retina_detect: true
});

//=================================================================================================

// <!-- Background Music -->
const audio = new Audio('effects/cryptonight.mp3');
audio.loop = true;
audio.volume = 0.15;

function startAudio() {
    audio.play().catch(() => { });
    document.body.removeEventListener('click', startAudio);
    document.body.removeEventListener('touchstart', startAudio);
}

document.body.addEventListener('click', startAudio);
document.body.addEventListener('touchstart', startAudio);

//=================================================================================================

// Twitter post
document.getElementById("shareTwitterBtn").addEventListener("click", async () => {
  // 1. Створюємо canvas
  const canvas = document.getElementById("shareCanvas");
  const ctx = canvas.getContext("2d");
  // Білий фон (поки без картинки з бази)
  ctx.fillStyle = "#ffffff";
  ctx.fillRect(0, 0, canvas.width, canvas.height);
  // 2. Текст позиції
  ctx.fillStyle = "#1a2a50";
  ctx.font = "bold 32px Arial";
  ctx.fillText("$8 @ 37.1K", 40, 100);
  ctx.fillStyle = "#00cc44";
  ctx.font = "bold 60px Arial";
  ctx.fillText("102X", 40, 180);
  // 3. Додаємо фрази бренду
  ctx.fillStyle = "#1a2a50";
  ctx.font = "24px Arial";
  ctx.fillText("I hedged my position like a pro on hedgeyour.fun", 40, 260);
  ctx.fillStyle = "#9ac31c";
  ctx.font = "22px Arial";
  ctx.fillText("Join me on hedgeyour.fun", 40, 310);
  // 4. Конвертуємо у зображення
  const imageURL = canvas.toDataURL("image/png");
  // 5. Створюємо лінк для Twitter
  const tweetText = encodeURIComponent("Join me at hedgeyour.fun");
  const tweetUrl = `https://twitter.com/intent/tweet?text=${tweetText}`;
  // Twitter не дозволяє напряму передати base64-картинку,
  // тому зображення потрібно буде завантажити на сервер або IPFS.
  // Для тесту відкриваємо твітер з текстом:
  window.open(tweetUrl, "_blank");
});
