"""Weekly overtime pay for hourly shifts (US federal rule: >40h/week at 1.5x)."""
from datetime import datetime, timedelta

def week_start(dt):
    # Workweek starts Sunday 00:00 local time.
    return (dt - timedelta(days=(dt.weekday() + 1) % 7)).replace(hour=0, minute=0, second=0, microsecond=0)

def weekly_pay(shifts, rate_cents):
    """shifts: list of (start: datetime, end: datetime). Returns total pay in cents."""
    weeks = {}
    for start, end in shifts:
        minutes = int((end - start).total_seconds() // 60)
        weeks.setdefault(week_start(start), 0)
        weeks[week_start(start)] += minutes
    total = 0
    for minutes in weeks.values():
        regular = min(minutes, 40 * 60)
        overtime = minutes - regular
        total += regular * rate_cents // 60 + overtime * rate_cents * 3 // 2 // 60
    return total
