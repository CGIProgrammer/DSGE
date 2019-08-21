import matplotlib.pyplot as plt
import numpy as np

dt = 1.0/60.0

timeline = []
values = []

time = 0.0
value = 0.0
while value<0.9 and time<10.0:
    timeline.append(time)
    values.append(value)
    time += dt
    value += dt

plt.plot(timeline, values)

plt.xlabel('Время (%d шагов)' % len(values))
plt.ylabel('Значение')
plt.title('Переход от нуля к единице')
plt.grid(True)
plt.show()
