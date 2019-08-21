from siberian_ctypes import Vector

BAT,ITEM,USEFUL,AMMO,WEAPON,THROWABLE = list(range(6))
BAT,PUMP,MAG = list(range(3))

PACK_SIZE = 0
SPEED = 1
BULLETS = 2
MASS = 3

ammos = {}
ammos['ammo_5.45x39'] = (50,350,1,0.0045)
ammos['ammo_12g'] = (50,350,10,0.045)

weaps = {}
weaps['wpn_ak12'] = {}
weaps['wpn_ak12']['cal']  = 'ammo_5.45x39'
weaps['wpn_ak12']['shot_time']  = 0.075
weaps['wpn_ak12']['mag_size']  = 30
weaps['wpn_ak12']['accuracy']  = 0.01
weaps['wpn_ak12']['reload_type']  = MAG
weaps['wpn_ak12']['recoil']  = 0.01

weaps['wpn_spas12'] = {}
weaps['wpn_spas12']['cal']  = 'ammo_12g'
weaps['wpn_spas12']['shot_time']  = 0.75
weaps['wpn_spas12']['mag_size']  = 9
weaps['wpn_spas12']['accuracy']  = 0.01
weaps['wpn_spas12']['reload_type']  = PUMP
weaps['wpn_spas12']['recoil']  = 0.01

weaps['wpn_glow_stick'] = {}
weaps['wpn_glow_stick']['cal']  = None
weaps['wpn_glow_stick']['shot_time']    = 0.75
weaps['wpn_glow_stick']['mag_size']     = 0
weaps['wpn_glow_stick']['accuracy']     = 0.0
weaps['wpn_glow_stick']['reload_type']  = BAT
weaps['wpn_glow_stick']['recoil']       = 0.01

SHOW,IDLE,WALK,RUN,ATTACK,ALT_ATTACK,HIDE,RELOAD,AIM,AIM_SHOT,SHOW_SOUND,SHOT_SOUND,RELOAD_SOUND,HIDE_SOUND,MUZZLE = list(range(15))

hands_animations = {}
hands_animations['wpn_ak12'] = [['ak12_hide',49,0],     #show
                              ['ak12_idle',0,4],        #idle
                              ['ak12_run',0,0],         #walk
                              ['ak12_run',0,10,30,40],  #run
                              ['ak12_shoot',0,9],       #attack
                              ['ak12_shoot',0,9],       #alternative attack
                              ['ak12_hide',0,49],       #hide
                              ['ak12_reload',0,99],     #reload
                              ['ak12_aim',0,10],        #aim
                              ['ak12_aim_shot',0,10],   #aim shot
                              'data/sounds/machinegunes/ak12_show.wav',
                              ['data/sounds/machinegunes/ak12_shot{}.wav'.format(i) for i in range(1,4)],
                              'data/sounds/machinegunes/ak12_reload.wav',
                              'data/sounds/machinegunes/ak12_hide.wav']
                  
hands_animations['wpn_spas12'] = [['SPAS12_All',85,76],             #show
                                  ['SPAS12_All',60,60],             #idle
                                  ['SPAS12_All',21,21],             #walk
                                  ['SPAS12_All',0,0],               #run
                                  ['SPAS12_All',0,10],              #attack
                                  ['SPAS12_All',1,17],              #alternative attack
                                  ['SPAS12_All',76,85],             #hide
                                  ['SPAS12_All',11,22,40,49,59],    #reload
                                  ['SPAS12_All',60,65],             #aim
                                  ['SPAS12_All',65,75],   #aim shot
                                   'data/sounds/machinegunes/ak12_show.wav',
                                  ['data/sounds/machinegunes/ak12_shot{}.wav'.format(i) for i in range(1,4)],
                                   'data/sounds/machinegunes/ak12_reload.wav',
                                   'data/sounds/machinegunes/ak12_hide.wav']

hands_animations['wpn_glow_stick'] = [['GlowStick', 0,10],      #show
                                      ['GlowStick',10,10],      #idle
                                      ['GlowStick',10,10],      #walk
                                      ['GlowStick',10,10],      #run
                                      ['GlowStick',10,29],      #hit
                                      ['GlowStick',10,29],      #hit2
                                      ['GlowStick',10, 0],
                                      None,
                                      None,
                                      None,
                                      '',       #show sound
                                      '',       #attack sound
                                      '',
                                      '']       #hide sound
