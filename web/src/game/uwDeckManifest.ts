/**
 * Explicit UWPro template identities, adapted from RETORETO/UnderWorldPro.
 * Source and permission record: docs/UWPRO_IMPORT_2026-10-21.md.
 * Identity review does not certify UWPro rule equivalence or add card admission.
 * Keep paths exact: never infer a mapping from a filename prefix at runtime.
 */
export type UwPlayerMapping = {
  sourcePath: string; targetId: string; name: string; kind: string; color: string; cost: number;
  permanent: readonly [number, number, number]; temporary: readonly [number, number, number];
  notes?: readonly string[]; quarantine?: string;
};
export type UwSocietyMapping = { sourcePath: string; targetId: string; name: string };
export const UW_SOURCE_SHA = '0d60c78b68697867697f20f0c171e41445735cc4';
// Original identity baseline; v2 corrects three icon records against the same original images.
export const UW_REVIEWED_TARGET_SHA = 'd98117d188cfc65f4e29fb757dfcf9accdd545e5';
export const UW_MANIFEST_VERSION = 'uwpro-deck-preview-v2-icon-corrections';
export const UW_PLAYER_MAPPINGS: readonly UwPlayerMapping[] = [
  {"sourcePath":"res://Cards/Data/PlayerCards/BQ/BQ028_shuigoushuqunmitan.tres","targetId":"BQ028","name":"水沟鼠群密探","kind":"character","color":"蓝","cost":2,"permanent":[0,0,0],"temporary":[1,0,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/BQ/BQ030_caidingguan.tres","targetId":"BQ030","name":"裁定官","kind":"character","color":"蓝","cost":2,"permanent":[0,0,0],"temporary":[0,0,1]},
  {"sourcePath":"res://Cards/Data/PlayerCards/BQ/BQ035_youmingqishi.tres","targetId":"BQ035","name":"幽冥骑士","kind":"character","color":"蓝","cost":5,"permanent":[0,2,1],"temporary":[1,0,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/BQ/BQ040_xianxue.tres","targetId":"BQ040","name":"鲜血","kind":"attachment","color":"蓝","cost":1,"permanent":[0,0,0],"temporary":[0,0,0],"notes":["非角色防御的空值/0/-1记法不同；保留当前目录定义。"]},
  {"sourcePath":"res://Cards/Data/PlayerCards/BQ/BQ078_zhuanyeqingliyuan.tres","targetId":"BQ078","name":"专业清理员","kind":"character","color":"黑","cost":2,"permanent":[0,0,1],"temporary":[0,1,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/BQ/BQ083_suomingkulou.tres","targetId":"BQ083","name":"索命骷髅","kind":"character","color":"黑","cost":4,"permanent":[0,1,1],"temporary":[0,0,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JC/JC029_xinshengxuezu.tres","targetId":"JC029","name":"新生血族","kind":"character","color":"蓝","cost":2,"permanent":[0,0,0],"temporary":[0,1,1],"notes":["本版已按同一原图纠正永久/临时图标分类；旧内核目录不自动导入。"]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JC/JC030_juxingbianfu.tres","targetId":"JC030","name":"巨型蝙蝠","kind":"character","color":"蓝","cost":3,"permanent":[0,2,0],"temporary":[0,0,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JC/JC031_huanlingshi.tres","targetId":"JC031","name":"唤灵师","kind":"character","color":"蓝","cost":3,"permanent":[0,0,1],"temporary":[1,0,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JC/JC032_xuezuzhanglao.tres","targetId":"JC032","name":"血族长老","kind":"character","color":"蓝","cost":5,"permanent":[1,1,2],"temporary":[0,0,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JC/JC035_chuanmeimozhou.tres","targetId":"JC035","name":"传媒魔咒","kind":"attachment","color":"蓝","cost":3,"permanent":[0,0,0],"temporary":[1,0,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JC/JC036_chuyong.tres","targetId":"JC036","name":"初拥","kind":"attachment","color":"蓝","cost":5,"permanent":[0,0,0],"temporary":[0,0,0],"notes":["非角色防御的空值/0/-1记法不同；保留当前目录定义。"]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JC/JC037_kelisidinaaifuluode.tres","targetId":"JC037","name":"克里斯蒂娜·爱弗罗德","kind":"character","color":"蓝","cost":6,"permanent":[0,2,2],"temporary":[0,0,1]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JC/JC041_jinpo.tres","targetId":"JC041","name":"禁婆","kind":"character","color":"蓝","cost":4,"permanent":[0,1,0],"temporary":[0,1,1]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JC/JC042_morixintu.tres","targetId":"JC042","name":"末日信徒","kind":"character","color":"红","cost":1,"permanent":[0,1,0],"temporary":[0,0,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JC/JC044_zainanshizhe.tres","targetId":"JC044","name":"灾难使者","kind":"character","color":"红","cost":3,"permanent":[1,1,1],"temporary":[0,0,0],"notes":["本版已按同一原图纠正永久/临时图标分类；旧内核目录不自动导入。"]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JC/JC045_zhongbaijisi.tres","targetId":"JC045","name":"钟摆祭司","kind":"character","color":"红","cost":4,"permanent":[1,2,0],"temporary":[0,0,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JC/JC046_wumiantuluzhe.tres","targetId":"JC046","name":"无面屠戮者","kind":"character","color":"红","cost":5,"permanent":[0,2,1],"temporary":[0,0,1]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JC/JC047_liehuozhongwei.tres","targetId":"JC047","name":"烈火重围","kind":"spell","color":"红","cost":3,"permanent":[0,0,0],"temporary":[0,0,0],"notes":["非角色防御的空值/0/-1记法不同；保留当前目录定义。"]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JC/JC049_shenyuanxiyu.tres","targetId":"JC049","name":"深渊细语","kind":"spell","color":"红","cost":2,"permanent":[0,0,0],"temporary":[0,0,0],"notes":["非角色防御的空值/0/-1记法不同；保留当前目录定义。"]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JC/JC050_sangzhonghuixiang.tres","targetId":"JC050","name":"丧钟回响","kind":"spell","color":"红","cost":5,"permanent":[0,0,0],"temporary":[0,0,0],"notes":["非角色防御的空值/0/-1记法不同；保留当前目录定义。"]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JC/JC052_zhongyandeshikoudi.tres","targetId":"JC052","name":"终焉的诗寇蒂","kind":"character","color":"红","cost":7,"permanent":[1,0,2],"temporary":[1,0,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JC/JC054_menggusiwangruchong.tres","targetId":"JC054","name":"蒙古死亡蠕虫","kind":"character","color":"红","cost":4,"permanent":[0,1,0],"temporary":[0,1,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JC/JC056_diqujingyuan.tres","targetId":"JC056","name":"地区警员","kind":"character","color":"灰","cost":2,"permanent":[0,1,1],"temporary":[0,0,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JC/JC057_jiankongyuan.tres","targetId":"JC057","name":"监控员","kind":"character","color":"灰","cost":2,"permanent":[0,0,0],"temporary":[1,0,1]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JC/JC058_jianyidexingjing.tres","targetId":"JC058","name":"坚毅的刑警","kind":"character","color":"灰","cost":3,"permanent":[1,1,1],"temporary":[0,0,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JC/JC059_anquanbaoweibumen.tres","targetId":"JC059","name":"安全保卫部门","kind":"character","color":"灰","cost":3,"permanent":[0,0,1],"temporary":[0,2,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JC/JC060_shenmixueguwen.tres","targetId":"JC060","name":"神秘学顾问","kind":"character","color":"灰","cost":3,"permanent":[1,0,1],"temporary":[0,0,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JC/JC061_weijichulitepaixiaozu.tres","targetId":"JC061","name":"危机处理特派小组","kind":"character","color":"灰","cost":5,"permanent":[1,2,1],"temporary":[0,0,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JC/JC063_jingchezhuibu.tres","targetId":"JC063","name":"警车追捕","kind":"spell","color":"灰","cost":2,"permanent":[0,0,0],"temporary":[0,0,0],"notes":["非角色防御的空值/0/-1记法不同；保留当前目录定义。","子类标签含义相同，事务前缀记法不同；保留当前目录定义。"]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JC/JC066_fengmopingselinna.tres","targetId":"JC066","name":"封魔瓶瑟琳娜","kind":"character","color":"灰","cost":5,"permanent":[0,0,2],"temporary":[1,0,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JC/JC069_jingquan.tres","targetId":"JC069","name":"警犬","kind":"character","color":"灰","cost":2,"permanent":[0,0,0],"temporary":[0,2,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JC/JC084_jietouhunhun.tres","targetId":"JC084","name":"街头混混","kind":"character","color":"黑","cost":1,"permanent":[0,1,0],"temporary":[0,0,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JC/JC085_fushengxingshi.tres","targetId":"JC085","name":"复生行尸","kind":"character","color":"黑","cost":2,"permanent":[0,1,0],"temporary":[0,0,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JC/JC086_qieshiren.tres","targetId":"JC086","name":"窃尸人","kind":"character","color":"黑","cost":3,"permanent":[0,2,1],"temporary":[0,0,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JC/JC087_gelunidezousike.tres","targetId":"JC087","name":"格鲁尼的走私客","kind":"character","color":"黑","cost":4,"permanent":[0,1,1],"temporary":[1,1,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JC/JC088_zhiyeshashou.tres","targetId":"JC088","name":"职业杀手","kind":"character","color":"黑","cost":3,"permanent":[0,1,0],"temporary":[0,1,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JC/JC089_duxuezuzhou.tres","targetId":"JC089","name":"毒血诅咒","kind":"attachment","color":"黑","cost":2,"permanent":[0,0,0],"temporary":[0,0,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JC/JC090_linghunqiyue.tres","targetId":"JC090","name":"灵魂契约","kind":"attachment","color":"黑","cost":2,"permanent":[0,0,0],"temporary":[0,0,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JC/JC091_mousha.tres","targetId":"JC091","name":"谋杀","kind":"spell","color":"黑","cost":3,"permanent":[0,0,0],"temporary":[0,0,0],"notes":["非角色防御的空值/0/-1记法不同；保留当前目录定义。"]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JC/JC092_silingzhaohuan.tres","targetId":"JC092","name":"死灵召唤","kind":"spell","color":"黑","cost":2,"permanent":[0,0,0],"temporary":[0,0,0],"notes":["非角色防御的空值/0/-1记法不同；保留当前目录定义。","子类标签含义相同，事务前缀记法不同；保留当前目录定义。"]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JC/JC093_yuanlingzhouren.tres","targetId":"JC093","name":"怨灵咒刃","kind":"attachment","color":"黑","cost":1,"permanent":[0,0,0],"temporary":[0,0,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JC/JC094_weidadetingbabadi.tres","targetId":"JC094","name":"威达的汀巴巴迪","kind":"character","color":"黑","cost":6,"permanent":[0,0,2],"temporary":[3,0,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JC/JC095_tanggeluni.tres","targetId":"JC095","name":"唐·格鲁尼","kind":"character","color":"黑","cost":7,"permanent":[0,2,1],"temporary":[3,1,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JC/JC096_agelipazhiquan.tres","targetId":"JC096","name":"阿格里帕之犬","kind":"character","color":"黑","cost":4,"permanent":[0,2,1],"temporary":[1,0,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JZ/JZ022_anyeyoulvezhe.tres","targetId":"JZ22","name":"暗夜游掠者","kind":"character","color":"蓝","cost":2,"permanent":[0,1,0],"temporary":[0,1,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JZ/JZ024_kadinazhuijiren.tres","targetId":"JZ24","name":"卡迪纳追迹人","kind":"character","color":"蓝","cost":4,"permanent":[1,2,0],"temporary":[0,0,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JZ/JZ027_hadashaaifuluode.tres","targetId":"JZ27","name":"哈达莎·爱弗罗德","kind":"character","color":"蓝","cost":6,"permanent":[1,2,0],"temporary":[0,0,1]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JZ/JZ030_moshilunzhe.tres","targetId":"JZ30","name":"末世论者","kind":"character","color":"红","cost":1,"permanent":[0,0,1],"temporary":[0,0,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JZ/JZ031_pojianzhemijiaoxianren.tres","targetId":"JZ31","name":"破茧者秘教线人","kind":"character","color":"红","cost":3,"permanent":[1,0,1],"temporary":[0,0,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JZ/JZ035_siweiyanmie.tres","targetId":"JZ35","name":"思维湮灭","kind":"spell","color":"红","cost":2,"permanent":[0,0,0],"temporary":[0,0,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JZ/JZ043_wuzhuangshimin.tres","targetId":"JZ43","name":"武装市民","kind":"character","color":"灰","cost":2,"permanent":[0,0,1],"temporary":[0,1,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JZ/JZ044_luoshanjixunjing.tres","targetId":"JZ44","name":"洛杉矶巡警","kind":"character","color":"灰","cost":3,"permanent":[0,1,1],"temporary":[0,1,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JZ/JZ045_jingyongzhishengji.tres","targetId":"JZ45","name":"警用直升机","kind":"character","color":"灰","cost":5,"permanent":[1,1,1],"temporary":[0,0,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JZ/JZ046_biantongdiao.tres","targetId":"JZ46","name":"狴犴铜雕","kind":"attachment","color":"灰","cost":2,"permanent":[0,0,1],"temporary":[0,0,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JZ/JZ048_jietoujiefei.tres","targetId":"JZ48","name":"街头劫匪","kind":"character","color":"黑","cost":1,"permanent":[0,1,0],"temporary":[0,0,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JZ/JZ049_panshanxingshi.tres","targetId":"JZ49","name":"蹒跚行尸","kind":"character","color":"黑","cost":1,"permanent":[0,0,0],"temporary":[0,2,0],"notes":["本版已按同一原图纠正永久/临时图标分类；旧内核目录不自动导入。"]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JZ/JZ050_muxueshishigui.tres","targetId":"JZ50","name":"墓穴食尸鬼","kind":"character","color":"黑","cost":2,"permanent":[0,1,0],"temporary":[0,0,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JZ/JZ054_atamasifengxianyishi.tres","targetId":"JZ54","name":"阿塔玛斯奉献仪式","kind":"spell","color":"黑","cost":2,"permanent":[0,0,0],"temporary":[0,0,0],"notes":["非角色防御的空值/0/-1记法不同；保留当前目录定义。"]},
  {"sourcePath":"res://Cards/Data/PlayerCards/JZ/JZ055_chuanqiluomu.tres","targetId":"JZ55","name":"传奇落幕","kind":"spell","color":"黑","cost":2,"permanent":[0,0,0],"temporary":[0,0,0],"notes":["非角色防御的空值/0/-1记法不同；保留当前目录定义。"]},
  {"sourcePath":"res://Cards/Data/PlayerCards/WM/WM023_jinzhidezhenzhu.tres","targetId":"WM023","name":"尽职的珍珠","kind":"character","color":"蓝","cost":4,"permanent":[0,0,1],"temporary":[0,0,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/WM/WM038_xintulaoyin.tres","targetId":"WM038","name":"信徒烙印","kind":"attachment","color":"红","cost":1,"permanent":[0,0,0],"temporary":[0,0,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/WM/WM056_zangwufanzi.tres","targetId":"WM056","name":"赃物贩子","kind":"character","color":"黑","cost":1,"permanent":[0,0,1],"temporary":[0,0,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/WM/WM059_yuyetufu.tres","targetId":"WM059","name":"雨夜屠夫","kind":"character","color":"黑","cost":2,"permanent":[0,1,0],"temporary":[0,0,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/WM/WM062_paolu.tres","targetId":"WM062","name":"跑路","kind":"spell","color":"黑","cost":2,"permanent":[0,0,0],"temporary":[0,0,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/XQ/XQ012_baozaoxuepu.tres","targetId":"XQ12","name":"暴躁血仆","kind":"character","color":"蓝","cost":2,"permanent":[0,0,0],"temporary":[0,1,1]},
  {"sourcePath":"res://Cards/Data/PlayerCards/XQ/XQ014_xianxue.tres","targetId":"XQ14","name":"鲜血","kind":"attachment","color":"蓝","cost":1,"permanent":[0,0,0],"temporary":[0,0,0],"notes":["非角色防御的空值/0/-1记法不同；保留当前目录定义。"]},
  {"sourcePath":"res://Cards/Data/PlayerCards/XQ/XQ016_fawubulvshituan.tres","targetId":"XQ16","name":"法务部律师团","kind":"character","color":"蓝","cost":2,"permanent":[0,0,1],"temporary":[0,0,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/XQ/XQ017_jipin.tres","targetId":"XQ17","name":"祭品","kind":"character","color":"红","cost":1,"permanent":[0,0,0],"temporary":[0,0,1]},
  {"sourcePath":"res://Cards/Data/PlayerCards/XQ/XQ018_jishiren.tres","targetId":"XQ18","name":"计时人","kind":"character","color":"红","cost":2,"permanent":[0,0,0],"temporary":[1,0,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/XQ/XQ023_jinshendebanshiyuan.tres","targetId":"XQ23","name":"谨慎的办事员","kind":"character","color":"灰","cost":1,"permanent":[0,0,1],"temporary":[0,0,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/XQ/XQ024_fangbaojingcha.tres","targetId":"XQ24","name":"防暴警察","kind":"character","color":"灰","cost":4,"permanent":[1,1,1],"temporary":[0,0,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/XQ/XQ025_jiankongtantou.tres","targetId":"XQ25","name":"监控探头","kind":"attachment","color":"灰","cost":2,"permanent":[0,0,0],"temporary":[1,0,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/XQ/XQ027_dingdianqingchuxingdong.tres","targetId":"XQ27","name":"定点清除行动","kind":"spell","color":"灰","cost":4,"permanent":[0,0,0],"temporary":[0,0,0],"notes":["非角色防御的空值/0/-1记法不同；保留当前目录定义。"]},
  {"sourcePath":"res://Cards/Data/PlayerCards/XQ/XQ036_shengjiachongdeqingliyuan.tres","targetId":"XQ36","name":"圣甲虫的清理员","kind":"character","color":"黑","cost":1,"permanent":[0,0,1],"temporary":[0,0,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/XQ/XQ037_yezonghuikanmenren.tres","targetId":"XQ37","name":"夜总会看门人","kind":"character","color":"黑","cost":2,"permanent":[0,1,0],"temporary":[0,0,0]},
  {"sourcePath":"res://Cards/Data/PlayerCards/XQ/XQ038_jinteyibolaxin.tres","targetId":"XQ38","name":"金特·易卜拉欣","kind":"character","color":"黑","cost":3,"permanent":[0,0,1],"temporary":[2,0,0]},
];
// Existing red/black/blue/gray societies plus the existing generic society only.
export const UW_SOCIETY_MAPPINGS: readonly UwSocietyMapping[] = [
  {"sourcePath":"res://Cards/Data/CouncilCards/JC/MSJC003_wangzuohui.tres","targetId":"MSJC03","name":"王座会"},
  {"sourcePath":"res://Cards/Data/CouncilCards/JC/MSJC004_mingzhongjiaopai.tres","targetId":"MSJC04","name":"鸣钟教派"},
  {"sourcePath":"res://Cards/Data/CouncilCards/JC/MSJC005_guojiajigou.tres","targetId":"MSJC05","name":"国家机构"},
  {"sourcePath":"res://Cards/Data/CouncilCards/JC/MSJC007_fangbeixulie.tres","targetId":"MSJC07","name":"方碑序列"},
  {"sourcePath":"res://Cards/Data/CouncilCards/JC/MSJC009_mishe.tres","targetId":"MSJC09","name":"秘社"},
];
