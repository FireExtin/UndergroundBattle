var allCards = {};
var activeCards = [];
var shown_count = 0;
var keywords = {};
var sort_indices;

var filter = {
  "str": _ => true,
  "cost": _ => true,
  "color": _ => true,
  "set": _ => true,
  "magic": _ => true,
  "typeb": _ => true,
}


$(document).ready(() => {
  $.getJSON("/cards/cards.json", (json) => {
    // this will show the info it in firebug console
    allCards = json;
    activeCards = Object.values(allCards);

    $('#ucq-count').text(`共查询到 ${activeCards.length} 张牌。`)
    sortActiveCards();
    showMoreCards();
  });
  $.getJSON("/public/docs/keywords.json", (json) => {
    keywords = json;
  });

  sort_indices = new Sortable(document.getElementById('card-sort-indecies'), {
    animation: 150,
    ghostClass: 'item-active',
    direction: 'horizontal',
    onUpdate: (_) => {
      deck.sortCards(sort_indices.toArray());
      deck.showCards();
    }
  });

  $('#bd-title-input').focus(() => {
    if ($('#bd-title-input').val() == "点此修改卡组名")
      $('#bd-title-input').val("");
  }).blur(() => {
    if ($('#bd-title-input').val() == "")
      $('#bd-title-input').val("点此修改卡组名");
  })

  $('#bd-author-input').focus(() => {
    if ($('#bd-author-input').val() == "点此修改作者")
      $('#bd-author-input').val("");
  }).blur(() => {
    if ($('#bd-author-input').val() == "")
      $('#bd-author-input').val("点此修改作者");
  })

});

$(window).scroll(() => {
  if ($(window).scrollTop() + $(window).height() >= $('#underworld-cards-query').height()) {
    showMoreCards();
  }
});

function sortActiveCards() {
  activeCards = activeCards.sort((a, b) => {
    if (a['deckcard'] != b['deckcard'])
      return b['deckcard'] - a['deckcard'];
    if (a['set'] != b['set']) {
      const setmap = {
        "基础": 0, "序曲": 1, "洛城": 2,
        "间奏": 3, "帷幕": 4, "星光": 5,
        "传奇": 6, "霸权": 7
      };
      return setmap[b['set']] - setmap[a['set']];
    }
    return a['set-id'] - b['set-id'];
  }
  );
}

function showMoreCards() {
  let ucq_cards = $('#ucq-cards');
  for (let i = 0; i < 30; i++) {
    if (shown_count >= activeCards.length) {
      $('#ucq-show').text('木有嘞！');
      return;
    }

    let card = activeCards[shown_count];
    let c_html = $('<li></li>').addClass('card')
      .css('background-image', `url('/cards/compressed/${card.id} ${card.name}.jpg'`)
      .attr('data-index', `${shown_count}`)
      .click(() => addCard(`${card.id}`));

    c_html.on('contextmenu', (e) => {
      showCardInfo(`${card.id}`);
      e.preventDefault()
    });

    ucq_cards.append(c_html);
    shown_count++;
  }
}

// cost filter
function costSelect(id) {
  let n = parseInt(id);

  for (var i = -1; i <= 7; i++) {
    if (i == n) $(`#filter-cost-button-${i}`).addClass('active');
    else $(`#filter-cost-button-${i}`).removeClass('active');
  }

  if (n == -1)
    filter.cost = _ => true
  else if (n == 7)
    filter.cost = card => card.cost && parseInt(card.cost) >= 7
  else
    filter.cost = card => card.cost && parseInt(card.cost) == n
}

// color filter
function colorSelect(id) {
  let n = {
    "任意色": 0, "黄": 1, "绿": 2, "蓝": 3, "红": 4,
    "灰": 5, "白": 6, "黑": 7, "紫": 8, "中立": 9,
  }[id];

  for (var i = 0; i <= 9; i++) {
    if (i == n) $(`#filter-color-button-${i}`).addClass('active');
    else $(`#filter-color-button-${i}`).removeClass('active');
  }

  if (n == 0) filter.color = (_ => true);
  else filter.color = (card => card.color && card.color == id);
}

// set filter
function setSelect(id) {
  let [n, name] = {
    "Any": [0, ""], "JC": [1, "基础"], "XQ": [2, "序曲"],
    "LC": [3, "洛城"], "JZ": [4, "间奏"],
    "WM": [5, "帷幕"], "XG": [6, "星光"],
    "CQ": [7, "传奇"], "BQ": [8, "霸权"],
  }[id];

  for (var i = 0; i <= 8; i++) {
    if (i == n) $(`#filter-set-button-${i}`).addClass('active');
    else $(`#filter-set-button-${i}`).removeClass('active');
  }

  if (n == 0) filter.set = (_ => true);
  else filter.set = (card => card.set == name);
}

function magicSelect(id) {
  let n = {
    "任意魔法": 0, "心灵": 1, "神圣": 2, "星辰": 3,
    "死亡": 4, "鲜血": 5, "无魔法": 6,
  }[id];

  for (var i = 0; i <= 6; i++) {
    if (i == n) $(`#filter-magic-button-${i}`).addClass('active');
    else $(`#filter-magic-button-${i}`).removeClass('active');
  }

  if (n == 0)
    filter.magic = _ => true;
  else if (n == 6)
    filter.magic = card => !card.magic || card.magic == "";
  else
    filter.magic = card => card.magic && card.magic == id;
}

function typeSelect(id) {
  let n = {
    "任意": 0, "角色": 1, "事务": 2, "附属": 3, "秘社": 4, "地区": 5, "任务": 6
  }[id];

  for (var i = 0; i <= 6; i++) {
    if (i == n) $(`#filter-type-button-${i}`).addClass('active');
    else $(`#filter-type-button-${i}`).removeClass('active');
  }

  if (n == 0)
    filter.typeb = _ => true
  else
    filter.typeb = card => card["basic-type"].includes(id)
}

function updateTextFilter() {
  var t = document.getElementById("uw-search").value;
  var words = t.split(" ");
  filter.str = (card) => {
    var b = true;
    for (var i = 0; i < words.length; i++) {
      b &= card["search"].includes(words[i]);
    }
    return b;
  };
}

function filterCards() {
  updateTextFilter();

  $('#ucq-cards').empty();
  activeCards = [];
  shown_count = 0;

  for (let k in allCards) {
    let card = allCards[k];

    if (card.istoken && !$('#opt-token').prop('checked'))
      continue;

    if (filter.str(card) &&
      filter.typeb(card) &&
      filter.cost(card) &&
      filter.color(card) &&
      filter.set(card) &&
      filter.magic(card)) {
      activeCards.push(card);
    }
  }

  $('#ucq-count').text(`共查询到 ${activeCards.length} 张牌。`)
  $('#ucq-show').text('点击查看更多');

  sortActiveCards();
  showMoreCards();
}

// Hide the card information block
$(document).mouseup(function (e) {
  if (e.button != 0) return;
  var cpib = $('#cp-info');
  if (!cpib.is(e.target) && cpib.has(e.target).length === 0) {
    $('#card-preview').fadeOut(300);
  }

  var ed = $('#export-dialog');
  if (ed.css('display') != "none" && !ed.is(e.target) && ed.has(e.target).length === 0) {
    $('#dialog-bg').fadeOut(300);
  }

  var id = $('#import-dialog');
  if (id.css('display') != "none" && !id.is(e.target) && id.has(e.target).length === 0) {
    $('#dialog-bg').fadeOut(300);
  }
});

// Show the card information block
function showCardInfo(id) {
  let card = allCards[id];

  // set card image
  $('#cp-card').css('background-image', `url("/cards/compressed/${card.id} ${card.name}.jpg")`)

  // set card info
  let info = $('#cp-info');
  info.empty();

  // info: name; set|id 
  info.append($(`<div class="name">${card.name}</div>`))
    .append($(`<div class="setname"> ${card.set} | ${card.id} </div>`));

  var item;

  // info: [color] type
  item = $('<div class="type"></div>')
  if (card.color) item.text(` ${card.color}${card.type} `);
  else item.text(` ${card.type} `);
  info.append(item);

  // info: [cost][lyl~]
  item = $('<div class="attr"></div>');
  if (card.cost) item.html(`费用：${card.cost} `);
  // if (card.lyl) card.lyl.forEach(lyl => item.append(`${lyl} `));
  if (card.lyl) item.append(`${card.lyl}`);
  info.append(item);

  // info: [dfc] [abl] [magic]
  item = $('<div class="attr"></div>');
  if (card.dfc)
    item.html(`防御：${card.dfc}&nbsp;&nbsp;`);
  if (card.abl && card.abl != "")
    item.append(`能力：${card.abl}&nbsp;&nbsp;`);
  if (card.magic && card.magic != "")
    item.append(`魔法：${card.magic}`);
  info.append(item);

  // info: [req] [sc]
  item = $('<div class="attr"></div>');
  if (card.req) item.html(`势力值：${card.req}&nbsp;&nbsp;`);
  if (card.sc) item.append(`分数：${card.sc}`);
  info.append(item);

  // info: [hands]
  item = $('<div class="attr"></div>');
  if (card.hands) item.html(`起始手牌数量：${card.hands}`);
  info.append(item);

  // info: text
  var text = card.text;
  if ($('#opt-kws').prop('checked') && card.keywords.length > 0) {
    text += "\n\n";
    for (let kw of card.keywords) {
      console.log(kw);
      text += '> ' + keywords[kw] + "\n";
    }
  }

  info.append($(`<textarea class="text" readonly>${text}</textarea>`));

  $('#card-preview').fadeIn(300); //.css('display', 'block');
}

function findCardByShortId(id) {
  for (var k in allCards) {
    if (k.split(" ")[0] == id)
      return k;
  }
  return id;
}

function filterSocieties() {
  $('#ucq-cards').empty();
  activeCards = [];
  shown_count = 0;

  for (k in allCards) {
    let card = allCards[k];

    if (card.istoken) continue;

    if (card["basic-type"] == "秘社") {
      activeCards.push(card);
    }

    $('#ucq-count').text(`共查询到 ${activeCards.length} 张秘社。`)
    $('#ucq-show').text('点击查看更多');

    sortActiveCards();
    showMoreCards();
  }
}


function filterCore() {
  $('#ucq-cards').empty();
  shown_count = 0;
  activeCards = [allCards['BQ043'], allCards['BQ044'], allCards['BQ045'], allCards['BQ046']];
  showMoreCards();

  $('#ucq-count').text(`共查询到 ${activeCards.length} 张核心牌。`)
  $('#ucq-show').text('点击查看更多');
}

class DeckBuilder {
  constructor() {
    this.society = "";
    this.cards = {};
    this.author = "佚名";
    this.title = "强大的卡组"
    this.hasCoreArea = false;
    this.core = "";
  }


  updateSociety(id) {
    if (!id || id == "") return;

    var card = allCards[id];
    $('#bd-society .bdc-color')
      .css('background-image', `url('/public/image/colors/r${card.color}.png')`);
    $('#bd-society .bdc-hand').text(card.hands);
    $('#bd-society .bdc-name').text(card.name);
    $('#bd-society')
      .css('background-image', `url('/cards/compressed/${id} ${card.name}.jpg')`)
      .css('background-position', 'right 10%');
    this.society = id;

    $('#bd-society').on('contextmenu', (e) => {
      showCardInfo(`${id}`);
      e.preventDefault();
    })

    // 龙之欢宴-鲜血
    if (id != 'MSBQ01' && this.cards['BQ040'] && this.cards['BQ040'] > 3) {
      this.cards['BQ040'] = 3;
      this.showCards('BQ040');
    }
    // 圣甲虫药业-无名尸体
    if (id != 'MSBQ03' && this.cards['XQ46'] && this.cards['XQ46'] > 3) {
      this.cards['XQ46'] = 3;
      this.showCards('XQ46');
    }
    // 修格斯
    if (id == 'MSBQ02') {
      this.hasCoreArea = true;
      this.core = "";
    } else {
      this.hasCoreArea = false;
      this.core = "";
    }

    this.showCards();
  }

  addCard(id) {
    var card = allCards[id];
    if (id in this.cards) {
      let n = this.cards[id];

      if (card.name.includes("无知路人") || card.name == "精神锚点")
        this.cards[id] = n + 1;
      // 龙之欢宴-鲜血
      else if (card.name == "鲜血" && this.society == "MSBQ01")
        this.cards[id] = n + 1;
      // 圣甲虫药业-无名尸体
      else if (card.name == "无名尸体" && this.society == "MSBQ03")
        this.cards[id] = n + 1;
      else if (card.text.includes("唯一") || n == 3)
        delete this.cards[id];
      else
        this.cards[id] = n + 1;
    } // 修格斯-核心角色
    else if (['BQ043', 'BQ044', 'BQ045', 'BQ046'].includes(id) && this.society == "MSBQ02") {
      this.core = card.id;
    } else {
      this.cards[id] = 1;
    };
    this.sortCards(sort_indices.toArray());
    this.showCards(id);
  }

  removeCard(id) {
    this.cards[id] -= 1;
    if (this.cards[id] == 0)
      delete this.cards[id];

    this.showCards(id);
  }

  sortCards(arr) {
    let ks = Object.keys(this.cards);
    ks.sort((k1, k2) => {
      let c1 = allCards[k1];
      let c2 = allCards[k2];

      let b2i = (b) => b ? -1 : 1;

      for (let i = 0; i < arr.length; ++i)
        if (c1[arr[i]] != c2[arr[i]])
          return b2i(c1[arr[i]] < c2[arr[i]]);

      return 0;
    })
    this.cards = ks.reduce((res, key) => (res[key] = this.cards[key], res), {})
  }

  getCount(cond = (_) => true) {
    let count = 0;
    for (let id in this.cards) {
      let card = allCards[id];
      if (cond(card)) { count += this.cards[id]; }
    }
    return count;
  }

  validationCheck() {
    if (!this.society || this.society == "")
      return [false, '你需要在卡组中加入一张秘社牌。'];

    if (this.getCount() < 50)
      return [false, '套牌至少包含五十张牌'];

    if (this.society == "MSJC01 帷幕守望") {
      let c = this.getCount((card) => card.color == "黄色")
      if (c < 25)
        return [false, `【帷幕守望】中黄色卡牌的数量不能少于25，当前为${c}。`];
    } else if (this.society == "MSJC02 猎魔人") {
      let c = this.getCount((card) => card.color == "绿色")
      if (c < 25)
        return [false, `【猎魔人】中绿色牌的数量不能少于25，当前为${c}。`];
    } else if (this.society == "MSJC03 王座会") {
      let c = this.getCount((card) => card.color == "蓝色")
      if (c < 25)
        return [false, `【王座会】中蓝色牌的数量不能少于25，当前为${c}。`];
    } else if (this.society == "MSJC04 鸣钟教派") {
      let c = this.getCount((card) => card.color == "红色")
      if (c < 25)
        return [false, `【鸣钟教派】中红色牌的数量不能少于25，当前为${c}。`];
    } else if (this.society == "MSJC05 国家机构") {
      let c = this.getCount((card) => card.color == "灰色")
      if (c < 25)
        return [false, `【国家机构】中灰色牌的数量不能少于25，当前为${c}。`];
    } else if (this.society == "MSJC06 圣贤") {
      let c = this.getCount((card) => card.color == "白色")
      if (c < 25)
        return [false, `【圣贤】中白色牌的数量不能少于25，当前为${c}。`];
    } else if (this.society == "MSJC07 方碑序列") {
      let c = this.getCount((card) => card.color == "黑色")
      if (c < 25)
        return [false, `【方碑序列】中黑色牌的数量不能少于25，当前为${c}。`];
    } else if (this.society == "MSJC08 梦境使者") {
      let c = this.getCount((card) => card.color == "紫色")
      if (c < 25)
        return [false, `【梦境使者】中紫色牌的数量不能少于25，当前为${c}。`];
    } else if (this.society == "MSJC10 貘组") {
      for (let id in this.cards) {
        let card = allCards[id];
        if (card.color != "黄色" && card.color != "中立" && card.magic != "星辰")
          return [false, `【貘组】只能包含黄色、中立和具有星辰魔法领域的卡牌，当前卡组中有“${card.name}”。`];
      }
    } else if (this.society == "MSJC11 S.P.T执行部") {
      for (let id in this.cards) {
        let card = allCards[id];
        if (!(card.color == "绿色" || card.color == "中立" || (card.abl.includes('B') && card.type.includes('人类'))))
          return [false, `【S.P.T执行部】只能包含绿色、中立和具有B的人类角色的卡牌，当前卡组中有“${card.name}”。`];
      }
    } else if (this.society == "MSJC12 爱弗罗德家族") {
      for (let id in this.cards) {
        let card = allCards[id];
        if (!(card.color == "蓝色" || card.color == "中立" || card.type.includes('吸血鬼') || card.type.includes('奴仆')))
          return [false, `【爱弗罗德家族】只能包含蓝色、中立和其他派系的吸血鬼或奴仆角色的卡牌，当前卡组中有“${card.name}”。`];
      }
    } else if (this.society == "MSJC13 计时者") {
      let c = this.getCount((card) => card.color != "红色" && card.color != "中立");
      if (c > 12)
        return [false, `【计时者】中非红色或中立卡牌的数量不能超过12张，当前为${c}。`];
    } else if (this.society == "MSJC14 哨兵") {
      for (let id in this.cards) {
        let card = allCards[id];
        if (!(card.color == "灰色" || card.color == "中立" || !card.magic || card.magic == ""))
          return [false, `【哨兵】只能包含灰色、中立和不含魔法图标的卡牌，当前卡组中有“${card.name}”。`];
      }
    } else if (this.society == "MSJC15 隐士") {
      let c = this.getCount((card) => card.color != "白色" && card.color != "中立");
      if (c > 12)
        return [false, `【隐士】中非白色或中立卡牌的数量不能超过12张，当前为${c}。`];
    } else if (this.society == "MSJC16 颂亡者") {
      for (let id in this.cards) {
        let card = allCards[id];
        if (!(card.color == "黑色" || card.color == "中立" || card.magic == "死亡"))
          return [false, `【颂亡者】只能包含黑色、中立和具有死亡魔法领域的卡牌，当前卡组中有“${card.name}”。`];
      }
    } else if (this.society == "MSJC17 梦魇族群") {
      return [null, `暂时无法为【梦魇族群】判定卡组合法性，请手动确认。`];
    } else if (this.society == "MSJZ01 猛禽战术小组") {
      return [null, `暂时无法为【猛禽战术小组】判定卡组合法性，请手动确认。`];
    } else if (this.society == "MSJZ02 盛夏王廷") {
      for (let id in this.cards) {
        let card = allCards[id];
        if (!(card.color == "绿色" || card.type.includes("仙灵")))
          return [false, `【盛夏王廷】只能包含绿色和仙灵卡牌，当前卡组中有“${card.name}”。`];
      }
    } else if (this.society == "MSJZ03 末日钟") {
      return [false, `【末日钟】不能作为初始秘社牌。`];
    } else if (this.society == "MSJZ04 破茧者教团") {
      for (let id in this.cards) {
        let card = allCards[id];
        if (!(card.color == "红色" || card.color == "紫色" || card.color == "中立"))
          return [false, `【破茧者教团】只能包含红色、紫色和中立卡牌，当前卡组中有“${card.name}”。`];
      }
    } else if (this.society == "MSLC01 联席会议观察员") {
      for (let id in this.cards) {
        let card = allCards[id];
        if (!(card.color == "黄色" || card.color == "中立"))
          return [false, `【联席会议观察员】只能包含黄色和中立卡牌，当前卡组中有“${card.name}”。`];
      }
    } else if (this.society == "MSLC02 加利福尼亚猎团") {
      for (let id in this.cards) {
        let card = allCards[id];
        if (!(card.color == "绿色" || card.color == "中立"))
          return [false, `【加利福尼亚猎团】只能包含绿色和中立卡牌，当前卡组中有“${card.name}”。`];
      }
    } else if (this.society == "MSLC03 洛杉矶警察局") {
      for (let id in this.cards) {
        let card = allCards[id];
        if (!(card.color == "灰色" || card.color == "中立"))
          return [false, `【洛杉矶警察局】只能包含灰色和中立卡牌，当前卡组中有“${card.name}”。`];
      }
    } else if (this.society == "MSLC04 圣公会") {
      for (let id in this.cards) {
        let card = allCards[id];
        if (!(card.color == "白色" || card.color == "中立"))
          return [false, `【圣公会】只能包含白色和中立卡牌，当前卡组中有“${card.name}”。`];
      }
    } else if (this.society == "MSLC05 劣地长老") {
      for (let id in this.cards) {
        let card = allCards[id];
        if (!(card.color == "白色" || card.color == "中立" || card.magic == "星辰"))
          return [false, `【劣地长老】只能包含白色、中立和具有星辰魔法领域的卡牌，当前卡组中有“${card.name}”。`];
      }
    } else if (this.society == "MSLC06 居民武装") {
      for (let id in this.cards) {
        let card = allCards[id];
        if (!(card.color == "中立"))
          return [false, `【居民武装】只能包含中立卡牌，当前卡组中有“${card.name}”。`];
      }
    } else if (this.society == "MSWM01 曼纽家族") {
      for (let id in this.cards) {
        let card = allCards[id];
        if (!(card.color == "蓝色" || card.color == "黄色" || card.color == "中立" ||
          card.type.includes('吸血鬼') || card.type.includes('阴魂')))
          return [false, `【曼纽家族】只能包含蓝色、黄色、中立和其他派系的吸血鬼或阴魂牌，当前卡组中有“${card.name}”。`];
      }
    } else if (this.society == "MSWM01 公众安全委员会") {
      for (let id in this.cards) {
        let card = allCards[id];
        if (!(card.color == "灰色" || card.color == "黄色" || card.color == "中立" ||
          card.type.includes('特工') || card.type.includes('媒体') || card.type.includes('互联网')))
          return [false, `【公众安全委员会】只能包含灰色、黄色、中立和其他派系的特工、媒体或互联网牌，当前卡组中有“${card.name}”。`];
      }
    } else if (this.society == "MSWM03 冷山帮") {
      let c = this.getCount((card) => card.color != "黑色" && card.color != "中立");
      if (c > 12)
        return [false, `【冷山帮】中非黑色或中立卡牌的数量不能超过12张，当前为${c}。`];
    } else if (this.society == "MSWM04 梦魇主母") {
      for (let id in this.cards) {
        let card = allCards[id];
        if (!(card.color == "紫色"))
          return [false, `【梦魇主母】只能包含紫色牌，当前卡组中有“${card.name}”。`];
      }
    } else if (this.society == "ZHMSJZ02 凛冬王廷") {
      return [false, `【凛冬王廷】（转换）不能作为初始秘社牌。`];
    }

    let tips = [
      '或许你的卡组没有问题吧。',
      '你这套牌会很卡手吧！',
      '为什么不试试在卡组中加入200张牌呢？',
      '用脚在桌子底下进行一场隐秘对抗。',
      '如果洛城重置，崇洋都被医治。',
      '我组了一套8色卡组，你打不过我。',
    ];

    return [true, tips[Math.floor(Math.random() * tips.length)]];
  }

  showCards(fade_id = null) {
    let l = $('#bd-card-list').empty();
    let fade_ele = null;

    if (this.hasCoreArea) {
      if (this.core != "") {
        let card = allCards[this.core];
        let color = $('<span class="bdc-color"> </span>').css
          ('background-image', `url('/public/image/colors/r${card.color}.png')`)
        let card_html = $(`<li class="bd-card" onclick="filterCore()"></li>`)
          .css('background-image', `url('/cards/compressed/${card.id} ${card.name}.jpg')`)
          .append(color)
          .append($(`<span class="number-attr bdc-cost"> ${card.cost} </span>`))
          .append($(`<span class="bdc-name"> ${card.name} </span>`))
          .append($(`<span class="bdc-card-count bdc-core-icon">  </span>`))
          .on('contextmenu', (e) => {
            showCardInfo(`${card.id}`);
            e.preventDefault();
          });
        l.append(card_html);

        if (fade_id && fade_id == card.id) fade_ele = card_html;
      } else {
        let card_html = $(`<li class="bd-card" onclick="filterCore()"></li>`)
          .append($(`<span class="bdc-color"> </span>`))
          .append($(`<span class="number-attr bdc-cost"> 0 </span>`))
          .append($(`<span class="bdc-name" style="font-style:italic"> 选择一张核心牌 </span>`))
          .append($(`<span class="bdc-card-count bdc-core-icon"> </span>`));
        l.append(card_html);

        if (fade_id && fade_id == "core") fade_ele = card_html;
      }
    }

    for (var id in this.cards) {
      let card = allCards[id];

      let color = $('<span class="bdc-color"> </span>').css
        ('background-image', `url('/public/image/colors/r${card.color}.png')`)
      let card_html = $(`<li class="bd-card" onclick="removeCard('${id}')"></li>`)
        .on('contextmenu', (e) => {
          showCardInfo(`${card.id}`);
          e.preventDefault();
        })
        .css('background-image', `url('/cards/compressed/${id} ${card.name}.jpg')`)
        .append(color)
        .append($(`<span class="number-attr bdc-cost"> ${card.cost} </span>`))
        .append($(`<span class="bdc-name"> ${card.name} </span>`))
        .append($(`<span class="bdc-card-count"> x${this.cards[id]} </span>`));
      l.append(card_html);

      if (fade_id && fade_id == id) fade_ele = card_html;
    }

    $('#bdc-count').text(`卡组共 ${Object.values(this.cards).reduce((a, b) => {
      return parseInt(a) + parseInt(b);
    }, 0)
      } 张牌。`);

    if (fade_ele) {
      fade_ele.hide();
      fade_ele.fadeIn(300);
    }

    let [res, info] = this.validationCheck();
    if (res == null) {
      $('#error-icon')
        .css('background-image', "url('/public/image/information.png')");
      $('#error-icon-mobile')
        .css('background-image', "url('/public/image/information.png')");
    } else if (res) {
      $('#error-icon')
        .css('background-image', "url('/public/image/check.png')");
      $('#error-icon-mobile')
        .css('background-image', "url('/public/image/check.png')");
    } else {
      $('#error-icon')
        .css('background-image', "url('/public/image/error.png')");
      $('#error-icon-mobile')
        .css('background-image', "url('/public/image/error.png')");
    }
    $('#error-icon').attr('data-hover', info);
    $('#error-icon-mobile').attr('data-hover', info);
  }

  updateTitle() {
    this.title = $('#bd-title-input').val();
    this.author = $('#bd-author-input').val();
  }

  getDOUFormat() {
    var output = `# 复制这段代码到 http://ymsj.fun/decks/ 查看卡组
title: ${this.title}
author: ${this.author}`;

    output += `
society: "${this.society}"
cards:`;

    for (var id in this.cards) {
      output += `
  ${id}: ${this.cards[id]}`;
    }

    return output;
  }

  getCODFormat() {
    var output = `<cockatrice_deck version="1">
  <deckname></deckname>
  <comments></comments>
  <zone name="side">
    <card number="1" price="0" name="DQJC107 沉没的废墟"/>
    <card number="1" price="0" name="DQJC108 佛罗伦萨"/>
    <card number="1" price="0" name="DQJC109 京都"/>
    <card number="1" price="0" name="DQJC110 伦敦"/>
    <card number="1" price="0" name="DQJC111 莫斯科"/>
    <card number="1" price="0" name="DQJC112 纽约"/>
    <card number="1" price="0" name="DQJC113 切尔诺贝利"/>
    <card number="1" price="0" name="DQJC114 上海"/>
    <card number="1" price="0" name="DQJC115 死者之城"/>
    <card number="1" price="0" name="DQJC116 香港"/>
    <card number="1" price="0" name="DQWM001 武陵源"/>
    <card number="1" price="0" name="DQWM002 埃里伯斯山"/>
    <card number="1" price="0" name="DQWM003 古城瓦莱塔"/>
    <card number="1" price="0" name="DQWM004 亚伯拉罕湖"/>
    <card number="1" price="0" name="DQWM005 洛杉矶"/>
    <card number="1" price="0" name="DQXQ01 拉斯维加斯"/>
    <card number="1" price="0" name="DQXQYZ02 裂口女"/>
    <card number="1" price="0" name="DQXQZY01 瘦长鬼影"/>
    <card number="3" price="0" name="ZHWM048 凛冬之风英迪拉"/>
    <card number="1" price="0" name="${this.society}"/>
    <card number="1" price="0" name="TK 先手标志"/>
  </zone>
  <zone name="main">`;

    for (var id in this.cards) {
      output += `
    <card number="${this.cards[id]}" price="0" name="${id}"/>`;
    }
    output += `
  </zone>
</cockatrice_deck>`;

    return output;
  }

  getBase64Format() {
    var cards = {};
    for (let id in this.cards) {
      cards[id] = this.cards[id];
    }
    let deck = {
      society: this.society,
      cards: cards,
      hasCoreArea: this.hasCoreArea,
      core: this.core,
    };
    return btoa(JSON.stringify(deck));
  }

  read(str, format) {
    if (format == "json") {
      let vj = JSON.parse(str);
      this.title = vj.title;
      this.author = vj.author;
      this.society = vj.society;
      this.cards = vj.cards;
      return this;
    }
    if (format == "yaml") {
      let vj = YAML.parse(str);
      this.title = vj.title;
      this.author = vj.author;
      this.society = vj.society;
      this.cards = vj.cards;
      return this;
    }
    if (format == "xml") {
      var parser = new DOMParser();
      var cod = parser.parseFromString(str, "text/xml");
      this.title = "*待补充*";
      this.author = "*待补充*";

      // generate society
      var side = cod.getElementsByTagName("zone")[0].getElementsByTagName("card");
      for (let i = 0; i < side.length; i++) {
        let name = side[i].attributes["name"].value;
        if (name.startsWith('MS')) {
          this.society = name;
          break;
        }
      }

      // generate cards
      var xcards = cod.getElementsByTagName("zone")[1].getElementsByTagName("card");
      this.cards = {};
      for (let i = 0; i < xcards.length; i++) {
        let name = xcards[i].attributes["name"].value;
        let number = xcards[i].attributes["number"].value;
        this.cards[name] = number;
      }
      return this;
    }
    if (format == "base64") {
      // let vj = YAML.parse(atob(str));
      let vj = JSON.parse(atob(str));
      this.society = vj.society;
      this.cards = vj.cards;
      if (vj.hasCoreArea) {
        this.hasCoreArea = vj.hasCoreArea;
        this.core = vj.core;
      }

      // this.society = findCardByShortId(vj.society);
      // this.cards = {}
      // for (var k in vj.cards) {
      //   this.cards[findCardByShortId(k)] = vj.cards[k];
      // }
      return this;
    }
    if (format == "base64dou") {
      let vj = YAML.parse(atob(str));
      this.society = findCardByShortId(vj.society);
      this.cards = {}
      for (var k in vj.cards) {
        this.cards[findCardByShortId(k)] = vj.cards[k];
      }
      return this;
    }
  }
}

deck = new DeckBuilder()

function showMsgBox(s) {
  let mb = $('#message-box').text(s);
  mb.fadeIn(300);
  mb.fadeOut(900);
}

function addCard(id) {
  var card = allCards[id];

  if (card.istoken) {
    showMsgBox("指示物或衍生物不能加入牌组。")
    return;
  }

  if (card['basic-type'] == "地区") {
    showMsgBox("地区牌不能加入牌组。")
    return;
  }

  if (card['basic-type'] == "秘社") {
    deck.updateSociety(id);

    // 修格斯：筛选核心牌
    if (id == 'MSBQ02') {
      filterCore();
    }
    return;
  }

  deck.addCard(id)
}

function removeCard(id) {
  deck.removeCard(id);
}

function exportDeck() {
  deck.updateTitle();
  $('#import-dialog').hide();
  $('#export-dialog').show();
  $('#dialog-bg').fadeIn(300);

  let code = deck.getDOUFormat();
  $('#dou-codeblock').text(code);

  code = deck.getCODFormat();
  $('#cod-codeblock').text(code);

  code = deck.getBase64Format();
  $('#base64-codeblock').text(code);

  hljs.highlightAll();
}

function showTab(evt, method) {
  var i, tabcontent, tablinks;
  tabcontent = document.getElementsByClassName("tabcontent");
  for (i = 0; i < tabcontent.length; i++) {
    tabcontent[i].style.display = "none";
  }
  tablinks = document.getElementsByClassName("tablinks");
  for (i = 0; i < tablinks.length; i++) {
    tablinks[i].className = tablinks[i].className.replace(" active", "");
  }
  document.getElementById(method).style.display = "block";
  evt.currentTarget.className += " active";
}

function copyDeckCode(_, format) {
  var code;
  if (format == 'dou') {
    code = deck.getDOUFormat();
  } else if (format == "cod") {
    code = deck.getCODFormat();
  } else if (format == "base64") {
    code = deck.getBase64Format();
  }
  navigator.clipboard.writeText(code);
}

function importDeck() {
  deck.updateTitle();
  $('#import-dialog').show();
  $('#export-dialog').hide();
  $('#dialog-bg').fadeIn(300);
}

function checkType(str) {
  let s = str.trim();
  if (s[0] == '<') return "xml";
  else if (s[0] == '{') return "json";
  else if (s.substring(0, 4) == 'CnNv') return "base64dou";
  else if (s.substring(0, 4) == 'eyJz') return "base64";
  else return "yaml";
}

function impDeckCode() {
  let code = $('#code-input').val();
  let t = checkType(code);
  showMsgBox(`检测到${t}格式卡组。`);
  deck.read(code, t);
  deck.sortCards(sort_indices.toArray());
  deck.updateSociety(deck.society);
  deck.showCards();
  $('#bd-title-input').val(deck.title);
  $('#bd-author-input').val(deck.author);
  $('#dialog-bg').fadeOut(300);
}

$(window).bind('beforeunload', function () {
  if (Object.keys(deck.cards).length > 0)
    return '您有尚未编辑完的卡组，确认退出？';
});