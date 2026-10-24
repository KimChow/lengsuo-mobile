package com.lengsuo.mobile.ui.shell

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.DateRange
import androidx.compose.material.icons.filled.Home
import androidx.compose.material.icons.filled.Person
import androidx.compose.material.icons.filled.Search
import androidx.compose.ui.graphics.vector.ImageVector
import com.lengsuo.mobile.R

enum class MainTab(val labelRes: Int, val icon: ImageVector) {
    TASKS(R.string.tab_tasks, Icons.Filled.Home),
    SCHEDULED(R.string.tab_scheduled, Icons.Filled.DateRange),
    DISCOVER(R.string.tab_discover, Icons.Filled.Search),
    ME(R.string.tab_me, Icons.Filled.Person),
}
