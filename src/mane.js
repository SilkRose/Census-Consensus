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
	let int = setInterval(function () {
		let days = Math.round(seconds / 86_400);
		let hours = Math.round((seconds % 86_400) / 3_600);
		let minutes = Math.round((seconds % 3_600) / 60);
		let seconds_ = seconds % 60;
		let countdown = "";
		if (days > 0) {
			countdown += `${days} days, `
		}
		if (hours > 0) {
			countdown += `${hours} hours, `
		}
		if (minutes > 0) {
			countdown += `${minutes} minutes, `
		}
		if (seconds_ > 0) {
			countdown += `${seconds_} seconds`
		}
		span.innerHTML = countdown;
		seconds--;
		if (seconds < 0) {
			clearInterval(int);
			span.innerHTML = message;
		}
	}, 1000)
}
