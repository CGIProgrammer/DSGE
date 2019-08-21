/*
 * animation.c
 *
 *  Created on: 15 янв. 2018 г.
 *      Author: ivan
 */

#include "engine.h"
#define ACTION_FPS 30

// Проверяет, воспроизводится ли на заданном слое анимация
// sActionStop(скелет, слой)
uint8_t sActionIsPlaying(sSkeleton* skeleton,uint8_t layer)
{
    return skeleton->action[layer].type!=ACTION_DUMMY && (skeleton->action[layer].type&0x0F)!=ACTION_STOPED;
}

// Останавливает анимацию, воспроизводящуюся на заданном слое
// sActionStop(скелет, слой)
void sActionStop(sSkeleton* skeleton,uint8_t layer)
{
    skeleton->action[layer].type = ACTION_DUMMY;
    skeleton->action[layer].time = skeleton->action[layer].time_end;
    skeleton->action[layer].time_begin = 0.0;
    skeleton->action[layer].time_end = 0.0;
    skeleton->action[layer].time_increment = 0.0;
    skeleton->action_weights[layer] = 0.0;
}

// Начинает воспроизведение анимации
// sActionPlay(скелет, слой, тип_воспроизведения, начальный_кадр, конечный_кадр, скорость)
void sActionPlay(sSkeleton* skeleton, uint8_t layer, uint32_t act_type, float start, float stop, float speed)
{
    sActionStop(skeleton,layer);
    if (start >= skeleton->action[layer].channels[0].keyframes_count) start = skeleton->action[layer].channels[0].keyframes_count-2;
    if (stop >= skeleton->action[layer].channels[0].keyframes_count) stop = skeleton->action[layer].channels[0].keyframes_count-2;
    if (start<0) start = 0.0;
    if (stop<0) stop = 0.0;
    //if (start==stop) return;
    skeleton->action[layer].type = act_type;
    skeleton->action[layer].time_increment = ((start<stop) ? fabs(speed) : -fabs(speed));
    skeleton->action[layer].time_begin = start;
    skeleton->action[layer].time_end = stop;
    skeleton->action[layer].time = start;
    skeleton->action_weights[layer] = 1.0;
    for (uint16_t i=0;i<skeleton->bone_count;i++)
    {
    	int start_frame = start;
    	float coeff = start - start_frame;
    	int ch_index = skeleton->action[layer].bones_matching[i];
    	if (ch_index==-1)
    	{
    		continue;
    	}
    	//sActionChannel* channel = skeleton->action[layer].channels + ch_index;
    	laType interpolation = Interpolate(skeleton->action[layer].channels[i].keyframes[start_frame],
    									   skeleton->action[layer].channels[i].keyframes[start_frame+1],
										   coeff);
    	skeleton->action[layer].channels[i].start_keyframe = Inverted(interpolation);
    }
    //action[l].bones_matching[i]
}

// Задаёт параметры (способ воспроизведения, начальный и конечный кадры и скорость)
// sActionSetParam(скелет, слой, тип_воспроизведения, начальный_кадр, конечный_кадр, скорость)
void sActionSetParam(sSkeleton* skeleton, uint8_t layer, int32_t act_type, float start, float stop, float speed)
{
    if (start >= skeleton->action[layer].channels[0].keyframes_count) start = skeleton->action[layer].channels[0].keyframes_count-2;
    if (stop >= skeleton->action[layer].channels[0].keyframes_count) stop = skeleton->action[layer].channels[0].keyframes_count-2;
    if (start<0) start = 0.0;
    if (stop<0) stop = 0.0;
    //if (start==stop) return;
    skeleton->action[layer].type = act_type;
    if (speed>0.0)
    {
    	skeleton->action[layer].time_increment = ((start<stop) ? fabs(speed) : -fabs(speed));
    }
    if (start>0.0)
    {
    	skeleton->action[layer].time_begin = start;
    	skeleton->action[layer].time = start;
    }
    if (stop>0.0)
    {
    	skeleton->action[layer].time_end = stop;
    }
    skeleton->action_weights[layer] = 1.0;
}

// Обрабатывает аимацию скелета. Двигает кости согласно заданным на слоях анимациям.
void sActionProcess(sSkeleton* skeleton)
{
    sAction* action = skeleton->action;
    for (index_t l=0;l<MAX_ACTION_LAYERS;l++)
    	if (action[l].type!=ACTION_DUMMY && (action[l].type&0x0F)!=ACTION_STOPED)
            action[l].time += action[l].time_increment*ACTION_FPS*sGetFrameTime();
    for (index_t i=0;i<skeleton->bone_count;i++)
    {
        if (!skeleton->bones[i].animated) continue;
        laType mat = skeleton->pose[i];

        float weights = 0.0;
        for (index_t l=0;l<MAX_ACTION_LAYERS;l++)
        {
            if (action[l].type==ACTION_DUMMY) continue;
            if (action[l].bones_matching[i] == -1)
            {
            	//printf("skip %s\n", skeleton->bones[i].name);
            	continue;
            }
            else
            {
            	//printf("animating %d\n", action[l].bones_matching[i]);
            }
            sActionChannel* channel = action[l].channels + action[l].bones_matching[i];
            float start_time = action[l].time_begin,
            end_time = action[l].time_end,
            time = action[l].time;
            /*if (time>=action[l].channels[i].keyframes_count-1)
            {
                time = action[l].channels[i].keyframes_count-1;
            }*/
            
            action[l].time = time;
            
            float time_begin = MIN(start_time,end_time);
            float time_end   = MAX(start_time,end_time);

            if ((action[l].type&0xF) == ACTION_PLAY  &&  (time>time_end || time<time_begin))
            {
            	action[l].type = (action[l].type&0xF0) | (ACTION_STOPED&0x0F);
            	action[l].time = time = (time>time_end) ? time_end : (time<time_begin ? time_begin : time);
            }
            else if ((action[l].type&0xF) == ACTION_LOOP)
            {
                if (time>MAX(start_time,end_time)) time=MIN(start_time,end_time);
                if (time<MIN(start_time,end_time)) time=MAX(start_time,end_time);
                action[l].time = time;
            }
            float framef = action[l].time;
            index_t framei = framef;
            float coeff = framef-framei;
            
            laType current_frame = Interpolate(channel->keyframes[framei],channel->keyframes[framei+1],coeff);
            
            if (action[l].type & 0x10)
            {
            	if (!memcmp(&mat,&Identity,sizeof(laType)))
            	{
            		mat = Inverted(channel->start_keyframe);
            	}
            	current_frame = Mul(channel->start_keyframe, current_frame);
            	current_frame = Interpolate(Identity, current_frame, skeleton->action_weights[l]);
            	mat = Mul(mat, current_frame);
            }
            else
            {
            	mat = Interpolate(mat,current_frame,skeleton->action_weights[l]);
            }
            weights += skeleton->action_weights[l];
        }
        skeleton->bones[i].transform = mat;
        if (mat.type!=16)
        {
        	puts("Wrong type");
        	exit(-1);
        }
    }
    
}

// Сбрасывает позу
void sSkeletonResetPose(sSkeleton* skeleton)
{
	for (uint32_t b=0;b<skeleton->bone_count;b++)
	{
		skeleton->pose[b] = Identity;
	}
}

// Добавляет позу из анимации на заданном слое к текущей позе
void sSkeletonAddPoseFromLayerToPose(sSkeleton* skeleton, uint8_t layer, float time, float weight)
{
	sAction* action = skeleton->action + layer;
	for (uint32_t b=0;b<skeleton->bone_count;b++)
	{
		sActionChannel* channel = action->channels + b;
		laType iKF = channel->start_keyframe;
		laType cf  = channel->keyframes[((uint32_t)time) + 0];
		laType nf  = channel->keyframes[((uint32_t)time) + 1];
		laType fr  = Interpolate(cf, nf, time-(uint32_t)time);
		fr = Mul(fr, iKF);
		fr = Interpolate(Identity, fr, weight);
		skeleton->pose[b] = Mul(fr, skeleton->pose[b]);
	}
}

// Смешивает текущую позу с позой из анимации на заданном слое
void sSkeletonMixPoseFromLayerWithPose(sSkeleton* skeleton, uint8_t layer, float time, float weight)
{
	sAction* action = skeleton->action + layer;
	for (uint32_t b=0;b<skeleton->bone_count;b++)
	{
		sActionChannel* channel = action->channels + b;
		laType cf  = channel->keyframes[((uint32_t)time) + 0];
		laType nf  = channel->keyframes[((uint32_t)time) + 1];
		laType fr  = Interpolate(cf, nf, time-(uint32_t)time);
		skeleton->pose[b] = Interpolate(skeleton->pose[b], fr, weight);
	}
}

// Добавляет позу из анимации к текущей позе
void sSkeletonAddPoseFromActionToPose(sSkeleton* skeleton, char* name, uint32_t keyframe, float time, float weight)
{
	sScene* scene = skeleton->scene;
	sAction* action = scene->actions;
	uint32_t i = 0;
	for (i = 0; i < scene->actions_count; i++)
	{
		action++;
		if (strcmp(action->name, name)==0)
		{
			break;
		}
	}

	if (i==scene->actions_count)
	{
		fprintf(LOGOUT, "Warning in sSkeletonAddPoseFromActionToPose: action %s not found\n", name);
		return;
	}

	for (uint32_t b=0;b<skeleton->bone_count;b++)
	{
		sActionChannel* channel = action->channels + b;
		laType iKF = Inverted(channel->keyframes[keyframe]);
		laType cf  = channel->keyframes[((uint32_t)time) + 0];
		laType nf  = channel->keyframes[((uint32_t)time) + 1];
		laType fr  = Interpolate(cf, nf, time-(uint32_t)time);
		fr = Mul(fr, iKF);
		fr = Interpolate(Identity, fr, weight);
		skeleton->pose[b] = Mul(fr, skeleton->pose[b]);
	}
}

// Смешивает позу из анимации с текущей позой
void sSkeletonMixPoseFromActionWithPose(sSkeleton* skeleton, char* name, uint32_t keyframe, float time, float weight)
{
	sScene* scene = skeleton->scene;
	sAction* action = scene->actions;
	uint32_t i = 0;
	for (i = 0; i < scene->actions_count; i++)
	{
		action++;
		if (strcmp(action->name, name)==0)
		{
			break;
		}
	}

	if (i==scene->actions_count)
	{
		fprintf(LOGOUT, "Warning in sSkeletonAddPoseFromActionToPose: action %s not found\n", name);
		return;
	}

	for (uint32_t b=0;b<skeleton->bone_count;b++)
	{
		sActionChannel* channel = action->channels + b;
		laType cf  = channel->keyframes[((uint32_t)time) + 0];
		laType nf  = channel->keyframes[((uint32_t)time) + 1];
		laType fr  = Interpolate(cf, nf, time-(uint32_t)time);
		skeleton->pose[b] = Interpolate(skeleton->pose[b], fr, weight);
	}
}

// Возвращает индекс ксти скелета
sBone* sSkeletonGetBoneByIndex(sSkeleton* skeleton,uint16_t ind)
{
    return skeleton->bones + ind;
}

// Возвращает количество костей скелета
uint16_t sSkeletonGetBoneCount(sSkeleton* skeleton)
{
    return skeleton->bone_count;
}

// Возвращает кость скелета по её имени
sBone* sSkeletonGetBone(sSkeleton* skeleton,char* name)
{
    for (uint32_t ind=0;ind<skeleton->bone_count;ind++)
    {
        if (!strcmp(name, skeleton->bones[ind].name))
        {
        	return &skeleton->bones[ind];
        }
    }
    printf("Warn: bone %s does not belong to %s\n",name,skeleton->name);
    return 0;
}

void sSkeletonSetAction(sSkeleton* skel,uint8_t layer,char* name)
{
    sScene* scene = skel->scene;
    uint32_t ind=0;
    for (ind=0;ind<scene->actions_count;ind++)
    {
        if (!strcmp(scene->actions[ind].name,name)) break;
    }
    if (ind<scene->actions_count)
    {
		sActionChannel* new_chs = scene->actions[ind].channels;
		sActionChannel* old_chs = skel->action[layer].channels;
		int* old_mch = skel->action[layer].bones_matching;

		skel->action[layer] = scene->actions[ind];

		if (!old_chs)
			old_chs = sCalloc(skel->bone_count, sizeof(sActionChannel));

		if (!old_mch)
			old_mch = sCalloc(skel->bone_count, sizeof(int));

		int bc = 0;
		for (int b=0; b<skel->bone_count; b++)
		{
			old_mch[b] = -1;
			for (int c=0; c<scene->actions[ind].channels_count; c++)
			{
				//printf("Marching %s, %s\n", skel->bones[b].name, new_chs[c].name);
				if (strcmp(skel->bones[b].name, new_chs[c].name)==0)
				{
					old_chs[bc] = new_chs[c];
					old_mch[b] = bc;
					bc++;
					break;
				}
			}
		}

    	//old_chs = sRealloc(old_chs, sizeof(sActionChannel[bc]));
		skel->action[layer].channels = old_chs;
    	skel->action[layer].bones_matching = old_mch;
    }
    else if (ind>=scene->actions_count)
    {
        fprintf(stderr,"Warning: action \"%s\" does not exists\nHere is a list of valid animation names:\n",name);
        for (ind=0;ind<scene->actions_count;ind++)
        {
            fprintf(stderr,"%s\n",scene->actions[ind].name);
        }
        return;
        
    }
    else if (skel->bone_count!=scene->actions[ind].channels_count || skel->name[0]!='s')
    {
        fprintf(stderr,"Warning: action \"%s\" is not compatible with object \"%s\"\n",name,skel->name);
        return;
    }
}

// Задаёт скелету анимацию по её имени
/*void sSkeletonSetAction(sSkeleton* skel,uint8_t layer,char* name)
{
    sScene* scene = skel->scene;
    uint32_t ind=0;
    for (ind=0;ind<scene->actions_count;ind++)
    {
        if (!strcmp(scene->actions[ind].name,name)) break;
    }
    if (ind<scene->actions_count)
    {
    	uint32_t bone_bytes_count = sizeof(sActionChannel[skel->bone_count]);
    	sActionChannel* oldArr = skel->action[layer].channels;
    	int* indArr = skel->action[layer].bones_matching;
    	skel->action[layer] = scene->actions[ind];

    	if (!oldArr) oldArr = sCalloc(bone_bytes_count, 1);
    	//if (!indArr) indArr = sMalloc(sizeof(indArr[skel->bone_count]));

    	memcpy(oldArr, scene->actions[ind].channels, bone_bytes_count);
    	skel->action[layer].channels = oldArr;
    }
    else if (ind>=scene->actions_count)
    {
        fprintf(stderr,"Warning: action \"%s\" does not exists\nHere is a list of valid animation names:\n",name);
        for (ind=0;ind<scene->actions_count;ind++)
        {
            fprintf(stderr,"%s\n",scene->actions[ind].name);
        }
        return;

    }
    else if (skel->bone_count!=scene->actions[ind].channels_count || skel->name[0]!='s')
    {
        fprintf(stderr,"Warning: action \"%s\" is not compatible with object \"%s\"\n",name,skel->name);
        return;
    }
}*/

// Возвращает скелет, которму принадлежит кость
sSkeleton *sBoneGetSkeleton(sBone* bone)
{
	return bone->skeleton;
}

// Возвращает кадр анимации, воспроизводящейся на задонном слое
float sSkeletonGetActionFrame(sSkeleton* skeleton,int layer)
{
	return skeleton->action[layer].time;
}

// Возвращает флаг анимации кости
int sBoneGetAnimatedFlag(sBone* bone)
{
	return bone->animated;
}

// Включает или отключает анимацию кости
void sBoneSetAnimatedFlag(sBone* bone, int flag)
{
	bone->animated = flag;
}

// Задаёт анимацию и её слой, способ воспроизведения, интервал и скорость.
void sSkeletonSetPlayAction(sSkeleton* skel,char* name,uint8_t layer, uint32_t act_type, float start, float stop, float speed)
{
	sSkeletonSetAction(skel,layer,name);
	sActionPlay(skel,layer,act_type,start,stop,speed);
}

// Задаёт кадр анимации, воспроизводящейся на заданном слое
void sSkeletonSetActionFrame(sSkeleton* skel, uint8_t layer,float frame)
{
	skel->action[layer].time_begin = 0.0;
	skel->action[layer].time_end   = skel->action[layer].channels[0].keyframes_count;
	if (frame>=skel->action[layer].time_end-1)
	{
		frame = skel->action[layer].time_end-1;
	}
	if (frame<0.0)
	{
		frame = 0.0;
	}
	skel->action[layer].time       = frame;
}

// Задаёт вес слоя анимации. Вес - коэффициент смешения анимационного слоя с предыдущим
void sSkeletonSetLayerWeight(sSkeleton* skel, uint8_t layer, float weight)
{
	if (layer<MAX_ACTION_LAYERS && skel->action[layer].type != ACTION_DUMMY)
	{
		skel->action_weights[layer] = weight>=0.0 ? (weight<=1.0 ? weight : 1.0) : 0.0;
	}
}

// Устанавливает время, воспроизводящейся анимации. Под временем понимается кадр анимации, умноженный на частоту кадров анимации
void sSkeletonSetLayerTime(sSkeleton* skel, uint8_t layer, float time)
{
	if (layer<MAX_ACTION_LAYERS && skel->action[layer].type != ACTION_DUMMY)
	{
		float interval = skel->action[layer].time_end - skel->action[layer].time_begin;
		float speed = interval/time / ACTION_FPS;
		float start = skel->action[layer].time_begin;
		float stop  = skel->action[layer].time_end;
		skel->action[layer].time_increment = ((start<stop) ? fabs(speed) : -fabs(speed));
	}
}

// Возвращает время, воспроизводящейся анимации. Под временем понимается кадр анимации, умноженный на частоту кадров анимации
float sSkeletonGetLayerTime(sSkeleton* skel, uint8_t layer)
{
	if (layer<MAX_ACTION_LAYERS && skel->action[layer].type != ACTION_DUMMY)
	{
		float interval = skel->action[layer].time_end - skel->action[layer].time_begin;
		float speed = skel->action[layer].time_increment;
		return interval/speed / ACTION_FPS;
	}
	else
	{
		return NAN;
	}
}

// Задаёт фазу анимации
void sSkeletonSetActionFrame2(sSkeleton* skel, uint8_t layer, float coeff)
{
	coeff -= (int)coeff;
	if (coeff<0.0)
	{
		coeff = 1.0 - coeff;
	}
	float s = skel->action[layer].time_begin;
	float e = skel->action[layer].time_end;
	skel->action[layer].time = s + coeff*(e-s);
}

// Возвращает фазу анимации
float sSkeletonGetActionFrame2(sSkeleton* skel, uint8_t layer)
{
	float s = skel->action[layer].time_begin;
	float e = skel->action[layer].time_end;
	float t = skel->action[layer].time;
	return (t - s) / (e - s);
}

// Задаёт скорость анимации, уже воспроизводящейся на заданном слое
void sSkeletonSetLayerSpeed(sSkeleton* skel, uint8_t layer, float speed)
{
	if (layer<MAX_ACTION_LAYERS && skel->action[layer].type != ACTION_DUMMY)
	{
		float start = skel->action[layer].time_begin;
		float stop  = skel->action[layer].time_end;
		skel->action[layer].time_increment = ((start<stop) ? fabs(speed) : -fabs(speed));
	}
}

// Возвращает скорость анимации, уже воспроизводящейся на заданном слое
float sSkeletonGetLayerSpeed(sSkeleton* skel, uint8_t layer)
{
	if (layer<MAX_ACTION_LAYERS && skel->action[layer].type != ACTION_DUMMY)
	{
		return skel->action[layer].time_increment;
	}
	else
	{
		return NAN;
	}
}

// Меняет слои анимации местами
void sSkeletonSwapLayers(sSkeleton* skeleton, uint16_t a, uint16_t b)
{
	sAction act = skeleton->action[a];
	skeleton->action[a] = skeleton->action[b];
	skeleton->action[b] = act;
}

// Задаёт конечный и начальный кадры анимации, уже воспроизводящейся на заданном слое
void sSkeletonSetActionInterval(sSkeleton* skeleton, uint8_t layer, float a, float b)
{
	sAction* act = skeleton->action + layer;
	float time = (act->time - act->time_begin) / (act->time_end - act->time_begin);
	act->time_begin = a;
	act->time_end   = b;
	act->time = time * (b-a) + a;
}

