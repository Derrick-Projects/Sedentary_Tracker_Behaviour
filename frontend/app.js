const CONFIG = {
    maxDataPoints: 100,
    thresholdFidget: 0.020,   // Matches Arduino THRESHOLD_FIDGET
    thresholdActive: 0.040,   // Matches Arduino THRESHOLD_ACTIVE  
    alertTimeSeconds: 1200,   // 20 minutes - matches Arduino ALERT_TIME_SEC
};


const state = {
    accelData: [],
    timelineData: [],
    totalReadings: 0,
    activeReadings: 0,
    longestInactive: 0,
    sedentaryTimer: 0,       
    currentState: 'SEDENTARY', // "ACTIVE", "FIDGET", or "SEDENTARY"
    alertCount: 0,
    alerts: [],
};


const elements = {
    connectionText: document.getElementById('connectionText'),
    statusDot: document.querySelector('.status-dot'),
    statusIndicator: document.getElementById('statusIndicator'),
    statusIcon: document.getElementById('statusIcon'),
    activityStateText: document.getElementById('activityStateText'),
    timerLabel: document.getElementById('timerLabel'),
    timerValue: document.getElementById('timerValue'),
    confidenceValue: document.getElementById('confidenceValue'),
    totalReadings: document.getElementById('totalReadings'),
    activePercentage: document.getElementById('activePercentage'),
    longestInactive: document.getElementById('longestInactive'),
    alertCount: document.getElementById('alertCount'),
    alertList: document.getElementById('alertList'),
};

const margin = { top: 20, right: 30, bottom: 30, left: 50 };

// Acceleration Chart
const accelChartEl = document.getElementById('accelChart');
const accelWidth = accelChartEl.clientWidth - margin.left - margin.right;
const accelHeight = 200;

const accelSvg = d3.select('#accelChart')
    .append('svg')
    .attr('width', accelWidth + margin.left + margin.right)
    .attr('height', accelHeight + margin.top + margin.bottom)
    .append('g')
    .attr('transform', `translate(${margin.left},${margin.top})`);

const accelX = d3.scaleLinear().domain([0, CONFIG.maxDataPoints - 1]).range([0, accelWidth]);
const accelY = d3.scaleLinear().domain([0, 0.1]).range([accelHeight, 0]);  // 0-0.1 range for acceleration delta

accelSvg.append('g')
    .attr('class', 'grid')
    .call(d3.axisLeft(accelY).tickSize(-accelWidth).tickFormat(''));

// FIDGET threshold (0.02)
accelSvg.append('line')
    .attr('class', 'threshold-line fidget-line')
    .attr('x1', 0)
    .attr('x2', accelWidth)
    .attr('y1', accelY(CONFIG.thresholdFidget))
    .attr('y2', accelY(CONFIG.thresholdFidget))
    .attr('stroke', '#eab308')
    .attr('stroke-dasharray', '5,5');

// ACTIVE threshold (0.04)
accelSvg.append('line')
    .attr('class', 'threshold-line active-line')
    .attr('x1', 0)
    .attr('x2', accelWidth)
    .attr('y1', accelY(CONFIG.thresholdActive))
    .attr('y2', accelY(CONFIG.thresholdActive))
    .attr('stroke', '#22c55e')
    .attr('stroke-dasharray', '5,5');

// Add axes
accelSvg.append('g')
    .attr('transform', `translate(0,${accelHeight})`)
    .attr('class', 'axis-label')
    .call(d3.axisBottom(accelX).ticks(5));

accelSvg.append('g')
    .attr('class', 'axis-label')
    .call(d3.axisLeft(accelY).ticks(5));

// Line generator
const accelLine = d3.line()
    .x((d, i) => accelX(i))
    .y(d => accelY(d))
    .curve(d3.curveMonotoneX);

// Area generator for fill
const accelArea = d3.area()
    .x((d, i) => accelX(i))
    .y0(accelHeight)
    .y1(d => accelY(d))
    .curve(d3.curveMonotoneX);

// Add area path
const areaPath = accelSvg.append('path')
    .attr('fill', 'rgba(59, 130, 246, 0.2)')
    .attr('stroke', 'none');

// Add line path
const linePath = accelSvg.append('path')
    .attr('fill', 'none')
    .attr('stroke', '#3b82f6')
    .attr('stroke-width', 2);

// Timeline Chart
const timelineChartEl = document.getElementById('timelineChart');
const timelineWidth = timelineChartEl.clientWidth - margin.left - margin.right;
const timelineHeight = 60;

const timelineSvg = d3.select('#timelineChart')
    .append('svg')
    .attr('width', timelineWidth + margin.left + margin.right)
    .attr('height', timelineHeight + margin.top + margin.bottom)
    .append('g')
    .attr('transform', `translate(${margin.left},${margin.top})`);

const timelineX = d3.scaleLinear().domain([0, CONFIG.maxDataPoints - 1]).range([0, timelineWidth]);

// Summary Chart
const summaryWidth = 200;
const summaryHeight = 200;
const summaryRadius = Math.min(summaryWidth, summaryHeight) / 2;

const summarySvg = d3.select('#summaryChart')
    .append('svg')
    .attr('width', summaryWidth)
    .attr('height', summaryHeight)
    .append('g')
    .attr('transform', `translate(${summaryWidth / 2},${summaryHeight / 2})`);

const arc = d3.arc()
    .innerRadius(summaryRadius - 30)
    .outerRadius(summaryRadius - 10);

const pie = d3.pie()
    .value(d => d.value)
    .sort(null);


let currentConnection = null;
let connectionType = null;
let reconnectAttempts = 0;
const MAX_RECONNECT_ATTEMPTS = 5;

function connect() {
    
    if (!currentConnection) {
        connectSSE();
    }
}

function connectSSE() {
    console.log('Attempting SSE connection...');
    connectionType = 'SSE';

    try {
        const eventSource = new EventSource(`${window.location.protocol}//${window.location.host}/events`);
        currentConnection = eventSource;

        eventSource.onopen = () => {
            elements.connectionText.textContent = 'Connected (SSE)';
            elements.statusDot.classList.remove('disconnected');
            elements.statusDot.classList.add('connected');
            console.log('SSE connected successfully');
            reconnectAttempts = 0;
        };

        eventSource.addEventListener('sensor-data', (event) => {
            processData(event.data);
        });

        eventSource.addEventListener('keepalive', (event) => {
            console.log('SSE keepalive received');
        });

        eventSource.onerror = (error) => {
            console.error('SSE error:', error);
            eventSource.close();
            currentConnection = null;

            // Fall back to WebSocket after SSE failure
            if (reconnectAttempts < MAX_RECONNECT_ATTEMPTS) {
                reconnectAttempts++;
                console.log(`SSE failed (attempt ${reconnectAttempts}), trying WebSocket fallback...`);
                setTimeout(() => connectWebSocket(), 1000);
            } else {
                elements.connectionText.textContent = 'Connection Failed';
                elements.statusDot.classList.remove('connected');
                elements.statusDot.classList.add('disconnected');
                console.error('Max reconnection attempts reached');
                // Reset after 30 seconds
                setTimeout(() => {
                    reconnectAttempts = 0;
                    connect();
                }, 30000);
            }
        };
    } catch (error) {
        console.error('Failed to create EventSource:', error);
        connectWebSocket();
    }
}

function connectWebSocket() {
    console.log('Attempting WebSocket connection...');
    connectionType = 'WebSocket';

    const ws = new WebSocket(`ws://${window.location.host}/ws`);
    currentConnection = ws;

    ws.onopen = () => {
        elements.connectionText.textContent = 'Connected (WebSocket)';
        elements.statusDot.classList.remove('disconnected');
        elements.statusDot.classList.add('connected');
        console.log('WebSocket connected');
        reconnectAttempts = 0;
    };

    ws.onmessage = (event) => {
        processData(event.data);
    };

    ws.onclose = () => {
        elements.connectionText.textContent = 'Disconnected';
        elements.statusDot.classList.remove('connected');
        elements.statusDot.classList.add('disconnected');
        console.log('WebSocket disconnected');
        currentConnection = null;

        // Retry connection
        if (reconnectAttempts < MAX_RECONNECT_ATTEMPTS) {
            reconnectAttempts++;
            console.log(`Reconnecting... (attempt ${reconnectAttempts})`);
            setTimeout(() => connect(), 3000);
        } else {
            console.error('Max reconnection attempts reached');
            // Reset after 30 seconds
            setTimeout(() => {
                reconnectAttempts = 0;
                connect();
            }, 30000);
        }
    };

    ws.onerror = (error) => {
        console.error('WebSocket error:', error);
    };
}

// Data Processing
function processData(rawData) {
    // Skip non-JSON lines
    if (!rawData.startsWith('{')) {
        return;
    }

    // Format: {"state":"SEDENTARY","timer":123,"val":0.015,"alert":false,"timestamp":"..."}
    let data;
    try {
        data = JSON.parse(rawData);
    } catch (e) {
        return;
    }

    // Handle error/debug messages from Arduino
    if (data.error) {
        return;
    }

    const activityState = data.state || 'SEDENTARY';  // "ACTIVE", "FIDGET", or "SEDENTARY"
    const timerSeconds = data.timer || 0;             // Sedentary timer from Rust
    const accelValue = data.val || 0;                 // Smoothed acceleration delta
    const alertTriggered = data.alert || false;       // Alert flag from backend

    state.totalReadings++;
    state.currentState = activityState;
    state.sedentaryTimer = timerSeconds;
    
    // Update data for acceleration chart
    state.accelData.push(accelValue);
    if (state.accelData.length > CONFIG.maxDataPoints) {
        state.accelData.shift();
    }
    
    // Track active readings for summary stats
    if (activityState === 'ACTIVE') {
        state.activeReadings++;
    }
    
    // Track longest inactive period
    if (timerSeconds > state.longestInactive) {
        state.longestInactive = timerSeconds;
    }
    
    // Trigger alert when backend signals it once per threshold crossing
    if (alertTriggered && timerSeconds % 60 === 0) {
        triggerAlert(timerSeconds);
    }

    // Timeline data: 0 = SEDENTARY, 1 = FIDGET, 2 = ACTIVE
    let activityLevel;
    switch (activityState) {
        case 'ACTIVE': activityLevel = 2; break;
        case 'FIDGET': activityLevel = 1; break;
        default: activityLevel = 0;  // SEDENTARY
    }
    state.timelineData.push(activityLevel);
    if (state.timelineData.length > CONFIG.maxDataPoints) {
        state.timelineData.shift();
    }

    // Update UI and charts
    updateUI();
    updateCharts();
}


function updateUI() {
    // Activity status - 3-state model
    const indicator = elements.statusIndicator;
    indicator.classList.remove('active', 'inactive', 'fidget');
    let iconHtml = '';
    switch (state.currentState) {
        case 'ACTIVE':
            indicator.classList.add('active');
            iconHtml = '<i class="fa-solid fa-bolt"></i>';
            elements.activityStateText.textContent = 'Active (Moving)';
            elements.timerLabel.textContent = 'Timer Reset!';
            break;
        case 'FIDGET':
            indicator.classList.add('fidget');
            iconHtml = '<i class="fa-solid fa-person-running"></i>';
            elements.activityStateText.textContent = 'Fidgeting (Paused)';
            elements.timerLabel.textContent = 'Timer Paused:';
            break;
        default:  // SEDENTARY
            indicator.classList.add('inactive');
            iconHtml = '<i class="fa-solid fa-couch"></i>';
            elements.activityStateText.textContent = 'Sedentary (Still)';
            elements.timerLabel.textContent = 'Sedentary for:';
    }
    elements.statusIcon.innerHTML = iconHtml;

    // Timer - show Arduino's sedentary timer
    const duration = state.sedentaryTimer;
    const minutes = Math.floor(duration / 60);
    const seconds = duration % 60;
    elements.timerValue.textContent = `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;

    // Confidence (based on data stability)
    const recentData = state.accelData.slice(-10);
    const avg = recentData.reduce((a, b) => a + b, 0) / recentData.length || 0;
    const variance = recentData.reduce((sum, val) => sum + Math.pow(val - avg, 2), 0) / recentData.length || 0;
    const confidence = Math.max(0, Math.min(100, 100 - variance * 50));
    elements.confidenceValue.textContent = confidence.toFixed(0);

    // Stats
    elements.totalReadings.textContent = state.totalReadings;
    const activePercent = state.totalReadings > 0 
        ? ((state.activeReadings / state.totalReadings) * 100).toFixed(1) 
        : 0;
    elements.activePercentage.textContent = `${activePercent}%`;
    elements.longestInactive.textContent = `${state.longestInactive}s`;
    elements.alertCount.textContent = state.alertCount;
}

function updateCharts() {
    // Acceleration chart
    if (state.accelData.length > 0) {
        linePath.datum(state.accelData).attr('d', accelLine);
        areaPath.datum(state.accelData).attr('d', accelArea);
    }

    // Timeline chart
    const barWidth = timelineWidth / CONFIG.maxDataPoints;
    
    const bars = timelineSvg.selectAll('rect')
        .data(state.timelineData);

    bars.enter()
        .append('rect')
        .merge(bars)
        .attr('x', (d, i) => timelineX(i))
        .attr('y', 0)
        .attr('width', barWidth - 1)
        .attr('height', timelineHeight)
        .attr('fill', d => {
            if (d === 2) return '#22c55e';  // ACTIVE - Green
            if (d === 1) return '#eab308';  // FIDGET - Yellow Timer Paused
            return '#ef4444';  // SEDENTARY - Red Timer Counting
        });

    bars.exit().remove();

    // Summary donut chart
    const summaryData = [
        { label: 'Active', value: state.activeReadings, color: '#22c55e' },
        { label: 'Inactive', value: state.totalReadings - state.activeReadings, color: '#ef4444' }
    ];

    const arcs = summarySvg.selectAll('path')
        .data(pie(summaryData));

    arcs.enter()
        .append('path')
        .merge(arcs)
        .attr('d', arc)
        .attr('fill', d => d.data.color)
        .attr('stroke', '#1e293b')
        .attr('stroke-width', 2);

    arcs.exit().remove();

    // Center text
    summarySvg.selectAll('text.center-text').remove();
    const activePercent = state.totalReadings > 0 
        ? ((state.activeReadings / state.totalReadings) * 100).toFixed(0) 
        : 0;
    
    summarySvg.append('text')
        .attr('class', 'center-text')
        .attr('text-anchor', 'middle')
        .attr('dy', '-0.2em')
        .attr('fill', '#f8fafc')
        .attr('font-size', '1.5rem')
        .attr('font-weight', 'bold')
        .text(`${activePercent}%`);

    summarySvg.append('text')
        .attr('class', 'center-text')
        .attr('text-anchor', 'middle')
        .attr('dy', '1.2em')
        .attr('fill', '#94a3b8')
        .attr('font-size', '0.8rem')
        .text('Active');
}

function triggerAlert(timerSeconds) {
    state.alertCount++;
    const now = new Date();
    const timeStr = now.toLocaleTimeString();
    
    state.alerts.unshift({
        time: timeStr,
        duration: timerSeconds
    });

    // Keep only last 10 alerts
    if (state.alerts.length > 10) {
        state.alerts.pop();
    }

    updateAlertList();
}

function updateAlertList() {
    if (state.alerts.length === 0) {
        elements.alertList.innerHTML = '<p class="no-alerts">No alerts yet</p>';
        return;
    }

    elements.alertList.innerHTML = state.alerts.map(alert => `
        <div class="alert-item">
            <span class="alert-time"> ${alert.time}</span>
            <span class="alert-duration">${alert.duration}s inactive</span>
        </div>
    `).join('');
}


// Decode JWT to get user info
function parseJwt(token) {
    try {
        const base64Url = token.split('.')[1];
        const base64 = base64Url.replace(/-/g, '+').replace(/_/g, '/');
        const jsonPayload = decodeURIComponent(atob(base64).split('').map(function(c) {
            return '%' + ('00' + c.charCodeAt(0).toString(16)).slice(-2);
        }).join(''));
        return JSON.parse(jsonPayload);
    } catch (e) {
        return null;
    }
}

// Setup user display and logout
function setupAuth() {
    const token = localStorage.getItem('token');
    if (token) {
        const payload = parseJwt(token);
        if (payload && payload.name) {
            document.getElementById('userName').textContent = payload.name;
        }
    }

    // Logout handler
    document.getElementById('logoutBtn').addEventListener('click', function() {
        localStorage.removeItem('token');
        window.location.href = '/login.html';
    });

    // Replay button handler
    document.getElementById('replayBtn').addEventListener('click', async function() {
        const btn = this;
        btn.disabled = true;
        btn.innerHTML = '<i class="fa-solid fa-spinner fa-spin"></i> Loading...';

        try {
            const response = await fetch('/api/replay');
            if (response.ok) {
                const text = await response.text();
                console.log('Replay started:', text);
                btn.innerHTML = '<i class="fa-solid fa-check"></i> Started';
                setTimeout(() => {
                    btn.disabled = false;
                    btn.innerHTML = '<i class="fa-solid fa-play"></i> Demo';
                }, 3000);
            } else {
                throw new Error('Failed to start replay');
            }
        } catch (error) {
            console.error('Replay error:', error);
            btn.innerHTML = '<i class="fa-solid fa-exclamation-triangle"></i> Error';
            setTimeout(() => {
                btn.disabled = false;
                btn.innerHTML = '<i class="fa-solid fa-play"></i> Demo';
            }, 3000);
        }
    });
}

function init() {
    setupAuth();
    connect();
    // Timer now driven by Arduino's inactiveSeconds - no local interval needed
}


init();
