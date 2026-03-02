(function () {
    'use strict';

    const form = document.getElementById('login-form');
    const submitBtn = document.getElementById('submit-btn');
    const btnText = document.getElementById('btn-text');
    const btnSpinner = document.getElementById('btn-spinner');
    const errorBanner = document.getElementById('error-banner');
    const errorText = document.getElementById('error-text');
    const pwInput = document.getElementById('password');
    const togglePw = document.getElementById('toggle-pw');
    const eyeOpen = document.getElementById('eye-open');
    const eyeClosed = document.getElementById('eye-closed');

    // ── Toggle password visibility ───────────────────────────────────────────
    togglePw.addEventListener('click', function () {
        const isPassword = pwInput.type === 'password';
        pwInput.type = isPassword ? 'text' : 'password';
        eyeOpen.classList.toggle('hidden', isPassword);
        eyeClosed.classList.toggle('hidden', !isPassword);
    });

    // ── Show / hide error banner ─────────────────────────────────────────────
    function showError(msg) {
        errorText.textContent = msg;
        errorBanner.classList.remove('hidden');
        // re-trigger shake animation
        errorBanner.style.animation = 'none';
        errorBanner.offsetHeight; // reflow
        errorBanner.style.animation = '';
    }

    function hideError() {
        errorBanner.classList.add('hidden');
    }

    // ── Set loading state ────────────────────────────────────────────────────
    function setLoading(loading) {
        submitBtn.disabled = loading;
        btnText.classList.toggle('hidden', loading);
        btnSpinner.classList.toggle('hidden', !loading);
    }

    // ── Form submission → POST /api/login ───────────────────────────────────
    form.addEventListener('submit', async function (e) {
        e.preventDefault();
        hideError();

        const username = document.getElementById('username').value.trim();
        const password = pwInput.value;

        if (!username || !password) {
            showError('Please enter your username and password.');
            return;
        }

        setLoading(true);

        try {
            const res = await fetch('/api/login', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ username, password }),
                credentials: 'same-origin',
            });

            const data = await res.json();

            if (res.ok && data.success) {
                // Redirect on success – adjust the destination as needed
                window.location.reload();
            } else {
                showError(data.message || 'Invalid credentials. Please try again.');
            }
        } catch (err) {
            showError('Network error. Please check your connection and try again.');
        } finally {
            setLoading(false);
        }
    });

    // Dismiss error on any input change
    form.addEventListener('input', hideError);
})();
