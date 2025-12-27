document.addEventListener('DOMContentLoaded', () => {
    console.log('Quadradical Dashboard Initialized');

    // Simulate some real-time updates
    const terminal = document.getElementById('terminal-body');
    const logs = [
        "quadradical::eval >> assessing risk for pending mutations...",
        "   optimizing context allocation for 73B behemoth.",
        "   analyzing crates/trinity-brain/src/main.rs for concurrency patterns.",
        "   system integrity: NOMINAL",
        "quadradical::think >> generating bridge logic for task streaming."
    ];

    let logIndex = 0;

    const addLog = (msg) => {
        const line = document.createElement('div');
        line.className = 'term-line prompt';
        line.innerText = msg;
        terminal.insertBefore(line, terminal.lastElementChild);
        terminal.scrollTop = terminal.scrollHeight;
    };

    setInterval(() => {
        if (Math.random() > 0.7) {
            addLog(logs[logIndex % logs.length]);
            logIndex++;
        }
    }, 4000);

    // Mock refreshing stats
    const totalTodos = document.getElementById('total-todos');
    let currentTotal = 507;

    setInterval(() => {
        if (Math.random() > 0.9) {
            currentTotal++;
            totalTodos.innerText = currentTotal;
        }
    }, 10000);
});
