function setTheme() {
	let radioSelectors = document.getElementsByName("theme");
	for (const radio in radioSelectors) {
		if (radio.checked) {
			return;
		}
	}
	let theme;
	for (const cookie of document.cookie.split(";")) {
		const [name, value] = cookie.trim().split("=");
		if (name === "theme") {
			theme = value;
		}
	}
	if (!theme) {
		const darkModeQuery = window.matchMedia('(prefers-color-scheme: dark)');
		if (darkModeQuery.matches) {
			theme = "dark";
		} else {
			theme = "light";
		}
	}
	if (theme === "light") {
		document.getElementById("light").checked = true;
		document.body.classList.add("light");
	} else if (theme === "dark") {
		document.getElementById("dark").checked = true;
		document.body.classList.add("dark");
	}
	updateTheme(theme);
}

function updateTheme(theme) {
	if (theme !== "light" && theme !== "dark") {
		return;
	}
	if (theme === "light") {
		document.body.classList.remove("dark");
		document.body.classList.add("light");
	} else if (theme === "dark") {
		document.body.classList.remove("light");
		document.body.classList.add("dark");
	}
	document.cookie = `theme=${theme};path=/;Max-Age=2592000;Secure=true;SameSite=Lax;`;
}

window.onload = setTheme;

function submitLogo(logo) {
	if (logo !== "census" && logo !== "consensus") {
		return;
	}
	fetch(`/logo/${logo}`, { method: 'POST' });
}

function countDown(seconds, message) {
	let span = document.getElementById("countdown");
	if (!span) return;
	let target = new Date(Date.now() + seconds * 1000);
	let int = setInterval(function () {
		let remaining = Math.max(0, (target - new Date()) / 1000);
		let days = Math.floor(remaining / 86400);
		let hours = Math.floor((remaining % 86400) / 3600);
		let minutes = Math.floor((remaining % 3600) / 60);
		let secs = Math.floor(remaining % 60);
		let parts = [];
		if (days) parts.push(format_plural(days, "day"));
		if (hours) parts.push(format_plural(hours, "hour"));
		if (minutes) parts.push(format_plural(minutes, "minute"));
		if (secs) parts.push(format_plural(secs, "second"));
		if (parts.length === 0) {
			parts.push("0 seconds");
		}
		span.innerHTML = parts.join(", ");
		if (remaining <= 0) {
			clearInterval(int);
			span.innerHTML = message;
		}
	}, 1000);
}

function format_plural(value, unit) {
	return `${value} ${unit}${value !== 1 ? "s" : ""}`;
}
