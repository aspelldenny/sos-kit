"""
Uptime monitor for Telegram bots (APScheduler integration).

Add to your bot's scheduler/jobs.py and register in scheduler/engine.py:

    from scheduler.jobs import job_uptime_monitor
    scheduler.add_job(
        job_uptime_monitor,
        IntervalTrigger(minutes=10),
        id="uptime_monitor",
        replace_existing=True,
    )

Requires: httpx, your bot's send_text() function, settings with telegram_chat_id.
"""

import httpx

# Track consecutive failures
_uptime_failures: dict[str, int] = {}

# Configure your services here
SERVICES = {
    "My App": "https://my-app.com",
    # "Staging": "https://staging.my-app.com",
}


async def job_uptime_monitor() -> None:
    """Mỗi 10 phút — ping services, alert if down."""
    try:
        async with httpx.AsyncClient(timeout=15.0) as client:
            for name, url in SERVICES.items():
                try:
                    resp = await client.get(url)
                    if resp.status_code == 200:
                        if _uptime_failures.get(name, 0) > 0:
                            await _send_alert(f"✅ {name} recovered ({url})")
                        _uptime_failures[name] = 0
                    else:
                        await _handle_failure(name, url, f"HTTP {resp.status_code}")
                except httpx.TimeoutException:
                    await _handle_failure(name, url, "timeout 15s")
                except httpx.ConnectError:
                    await _handle_failure(name, url, "connection refused")
                except Exception as e:
                    await _handle_failure(name, url, str(e))
    except Exception:
        pass  # Don't crash scheduler


async def _handle_failure(name: str, url: str, reason: str) -> None:
    """Alert on 1st failure, then every 6th (= every hour at 10min interval)."""
    _uptime_failures[name] = _uptime_failures.get(name, 0) + 1
    count = _uptime_failures[name]

    if count == 1 or count % 6 == 0:
        await _send_alert(
            f"🚨 {name} DOWN!\nURL: {url}\nReason: {reason}\nFailure #{count}"
        )


async def _send_alert(message: str) -> None:
    """Send alert via Telegram. Replace with your bot's send function."""
    # Example for python-telegram-bot:
    # from services.telegram import send_text
    # from config.settings import get_settings
    # await send_text(get_settings().telegram_chat_id, message)
    raise NotImplementedError("Replace with your bot's send_text()")
