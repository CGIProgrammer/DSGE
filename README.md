# DSGE
Легковесный графический движок для экспериментов с различными алгоритмами рендеринга.
Возможно, в будущем это выльется во что-то более-менее юзабельное.

Уже есть:
- Нодовая структура постобработки (пока ноды представлены толко в виде кода)
- Загрузка текстур для большинства известныз форматов:
    * обычныe картинки: jpeg, png и т.д. всё, что поддерживает крейт image
    * сжатые текстуры: ktx и dds
- Загрузка полисеток в собственном формате
- Интеграция с Blender. Пока только экспорт полисеток.

Чего нет, но хотелось бы добавить:
- освещение и тени,
- PBR,
- воксельный рендеринг,
- скелетную аниманию,
- волюметрики,
- временное сглаживание,
- временной суперсэмплинг,
- интерполяцию кадров (удвоение частоты кадров)
- зеркальные и рассеянные отражения в пространстве кадра (SSRT),
- полноценное совмещение марширующих лучей (ray marching) с полигональной геометрией.