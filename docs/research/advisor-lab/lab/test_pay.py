import unittest
from datetime import datetime as D
from pay import weekly_pay

class T(unittest.TestCase):
    def test_simple(self):
        self.assertEqual(weekly_pay([(D(2026,10,5,9), D(2026,10,5,17))], 2000), 16000)
    def test_overtime(self):
        s = [(D(2026,10,5+i,8), D(2026,10,5+i,18)) for i in range(5)]  # 50h
        self.assertEqual(weekly_pay(s, 2000), 40*2000 + 10*3000)
    def test_shift_crossing_week_boundary(self):
        # 40h already Sun-Fri, then Saturday 20:00 -> Sunday 04:00 (4h in old week, 4h in new week)
        s = [(D(2026,10,4+i,8), D(2026,10,4+i,16)) for i in range(5)]
        s.append((D(2026,10,10,20), D(2026,10,11,4)))
        self.assertEqual(weekly_pay(s, 2000), 40*2000 + 4*3000 + 4*2000)
    def test_fractional_cents(self):
        # 9 minutes at $19.99/h -> 299.85 cents -> 300 (round half up, once per week)
        self.assertEqual(weekly_pay([(D(2026,10,5,9), D(2026,10,5,9,9))], 1999), 300)

if __name__ == "__main__":
    unittest.main()
